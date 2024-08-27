use super::{chunk::Chunk, setting::Setting};

pub struct Terrain {
    setting: Setting
}

impl Terrain {

    pub async fn load(name: &str) -> anyhow::Result<Self> {
        let path = format!("pack/map/{name}");
        let setting = Setting::read(&path).await?;
        for x in 0..setting.map_size[0] {
            for y in 0..setting.map_size[1] {
                let name = Chunk::name_from(x, y);
                Chunk::new(&path, &name, &setting).await?;
            }
        }
        Ok(Self {
            setting
        })
    }

}

