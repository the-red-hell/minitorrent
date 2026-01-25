#[allow(async_fn_in_trait)]
pub trait FileSystem {
    type Error: defmt::Format;

    /// opens a file with ReadWriteCreateOrAppend mode
    fn open_file(&mut self, file_name: &str) -> Result<(), Self::Error>;

    fn open_dir(&mut self, dir_name: &str) -> Result<(), Self::Error>;

    async fn write_to_opened_file(&self, buf: &[u8]) -> Result<(), Self::Error>;

    async fn read_to_end(&self, buf: &mut [u8]) -> Result<usize, Self::Error>;
}
