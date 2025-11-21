use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine};
use octocrab::Octocrab;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct PublicKey {
    key_id: String,
    key: String,
}

#[derive(Debug, Deserialize)]
pub struct SecretInfo {
    #[serde(skip)]
    #[allow(dead_code)]
    name: String,
    #[serde(rename = "updated_at")]
    pub updated_at: Option<String>,
}

pub struct GitHubClient {
    octocrab: Octocrab,
    owner: String,
    repo: String,
}

impl GitHubClient {
    pub fn new(token: String, owner: String, repo: String) -> Result<Self> {
        let octocrab = Octocrab::builder()
            .personal_token(token)
            .build()
            .context("Failed to create Octocrab client")?;

        Ok(Self {
            octocrab,
            owner,
            repo,
        })
    }

    async fn get_public_key(&self) -> Result<PublicKey> {
        let path = format!(
            "/repos/{}/{}/actions/secrets/public-key",
            self.owner, self.repo
        );

        let public_key: PublicKey = self
            .octocrab
            .get(path, None::<&()>)
            .await
            .context("Failed to get public key from GitHub")?;

        Ok(public_key)
    }

    /// Encrypt secret value using NaCl sealed box encryption with repository's public key.
    /// Uses X25519-XSalsa20-Poly1305 as required by GitHub API.
    /// Sealed box automatically handles nonce and ephemeral key generation.
    /// Returns base64-encoded encrypted data.
    fn encrypt_secret(&self, public_key: &str, secret_value: &str) -> Result<String> {
        use sodoken::crypto_box;

        let public_key_bytes = general_purpose::STANDARD
            .decode(public_key)
            .context("Failed to decode public key")?;

        if public_key_bytes.len() != crypto_box::XSALSA_PUBLICKEYBYTES {
            anyhow::bail!(
                "Invalid public key length. Expected {} bytes, got {}",
                crypto_box::XSALSA_PUBLICKEYBYTES,
                public_key_bytes.len()
            );
        }

        let mut repository_public_key = [0u8; crypto_box::XSALSA_PUBLICKEYBYTES];
        repository_public_key.copy_from_slice(&public_key_bytes);

        // Encrypt using sealed box (automatically generates ephemeral key and nonce)
        let secret_bytes = secret_value.as_bytes();
        let mut encrypted = vec![0u8; secret_bytes.len() + crypto_box::XSALSA_SEALBYTES];
        
        crypto_box::xsalsa_seal(
            &mut encrypted,
            secret_bytes,
            &repository_public_key,
        )?;

        Ok(general_purpose::STANDARD.encode(&encrypted))
    }

    /// Retrieve information about a secret, including last update timestamp.
    /// Returns None if the secret doesn't exist.
    pub async fn get_secret_info(&self, secret_name: &str) -> Result<Option<SecretInfo>> {
        let path = format!(
            "/repos/{}/{}/actions/secrets/{}",
            self.owner, self.repo, secret_name
        );

        match self.octocrab.get::<SecretInfo, _, _>(path, None::<&()>).await {
            Ok(secret_info) => Ok(Some(secret_info)),
            Err(octocrab::Error::GitHub { source, .. }) if source.status_code == 404 => Ok(None),
            Err(e) => Err(anyhow::anyhow!("{}", e)).context("Failed to get secret info"),
        }
    }

    pub async fn update_secret(
        &self,
        secret_name: &str,
        secret_value: &str,
    ) -> Result<()> {
        let public_key = self.get_public_key().await?;
        let encrypted_value = self
            .encrypt_secret(&public_key.key, secret_value)
            .context("Failed to encrypt secret")?;

        #[derive(Serialize)]
        struct UpdateSecretRequest {
            encrypted_value: String,
            key_id: String,
        }

        let path = format!(
            "/repos/{}/{}/actions/secrets/{}",
            self.owner, self.repo, secret_name
        );

        let body = UpdateSecretRequest {
            encrypted_value,
            key_id: public_key.key_id,
        };

        // GitHub API returns 204 No Content on success (empty body)
        // Try to parse as JSON, but if it's empty (EOF error), that's also success
        match self.octocrab.put::<serde::de::IgnoredAny, _, _>(path, Some(&body)).await {
            Ok(_) => {
                // Success - response parsed (even if empty)
            }
            Err(octocrab::Error::Json { source, .. }) => {
                // JSON parsing error - check if it's due to empty response (204 No Content)
                // Empty responses mean success for PUT requests that return 204
                let error_msg = source.to_string();
                if error_msg.contains("EOF") || error_msg.contains("expected value") {
                    // Empty response (204 No Content) - this is success
                    // The secret was updated successfully
                } else {
                    // Actual JSON parsing error
                    return Err(anyhow::anyhow!("JSON parsing error: {}", source));
                }
            }
            Err(e) => {
                // Extract detailed error information for other errors
                let error_details = match &e {
                    octocrab::Error::GitHub { source, .. } => {
                        let mut msg = format!(
                            "GitHub API error (status {}): {}",
                            source.status_code,
                            source.message
                        );
                        if let Some(errs) = &source.errors {
                            if !errs.is_empty() {
                                msg.push_str(&format!(" Details: {:?}", errs));
                            }
                        }
                        if let Some(doc_url) = &source.documentation_url {
                            msg.push_str(&format!(" Documentation: {}", doc_url));
                        }
                        anyhow::anyhow!("{}", msg)
                    }
                    octocrab::Error::Http { source, .. } => {
                        anyhow::anyhow!("HTTP error: {}", source)
                    }
                    octocrab::Error::Uri { source, .. } => {
                        anyhow::anyhow!("URI error: {}", source)
                    }
                    _ => anyhow::anyhow!("{}", e),
                };
                return Err(error_details);
            }
        }

        Ok(())
    }
}
