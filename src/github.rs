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
    pub name: String,
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

    /// Encrypt secret value using NaCl box encryption with repository's public key.
    /// Returns base64-encoded encrypted data in format: [nonce][ephemeral_public_key][encrypted_data]
    fn encrypt_secret(&self, public_key: &str, secret_value: &str) -> Result<String> {
        use crypto_box::{PublicKey, SecretKey, ChaChaBox, aead::Aead};
        use rand_core::OsRng;
        use rand::RngCore;

        let public_key_bytes = general_purpose::STANDARD
            .decode(public_key)
            .context("Failed to decode public key")?;

        if public_key_bytes.len() != 32 {
            anyhow::bail!(
                "Invalid public key length. Expected 32 bytes, got {}",
                public_key_bytes.len()
            );
        }

        let public_key_array: [u8; 32] = public_key_bytes
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid public key length"))?;
        let repository_public_key = PublicKey::from(public_key_array);

        // Generate ephemeral key pair for this encryption session
        let ephemeral_secret = SecretKey::generate(&mut OsRng);
        let ephemeral_public = ephemeral_secret.public_key();

        let cipher = ChaChaBox::new(&repository_public_key, &ephemeral_secret);

        // Generate random nonce for encryption
        let mut nonce_bytes = [0u8; 24];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = crypto_box::Nonce::from(nonce_bytes);

        let encrypted = cipher
            .encrypt(&nonce, secret_value.as_bytes())
            .map_err(|_| anyhow::anyhow!("Failed to encrypt secret"))?;

        // Combine components: nonce (24 bytes) + ephemeral public key (32 bytes) + encrypted data
        let mut combined = Vec::new();
        combined.extend_from_slice(&nonce_bytes);
        combined.extend_from_slice(ephemeral_public.as_bytes());
        combined.extend_from_slice(&encrypted);

        Ok(general_purpose::STANDARD.encode(&combined))
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

        self.octocrab
            .put::<(), _, _>(path, Some(&body))
            .await
            .context("Failed to update secret")?;

        Ok(())
    }
}
