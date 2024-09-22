use super::weapon::Weapon;

pub struct Attachments {
    pub weapon: Weapon,
}

impl Attachments {
    pub async fn new() -> Self {
        Self {
            weapon: Weapon::None,
        }
    }
}