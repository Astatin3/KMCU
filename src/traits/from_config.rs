pub trait FromConfig {
    type ConfigType;

    fn from_config(config: Self::ConfigType) -> anyhow::Result<Self>
    where
        Self: Sized;
}
