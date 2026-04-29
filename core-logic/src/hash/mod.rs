#[allow(async_fn_in_trait)]
pub trait Sha1Hasher {
    /// Process data.
    async fn update(&self, data: &[u8]);

    /// Extract the final hash.
    async fn finalize(&self, digest: &mut [u8; 20]);
}
