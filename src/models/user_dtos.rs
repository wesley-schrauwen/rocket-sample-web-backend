use rocket::serde::{Deserialize, Serialize};
use crate::models::user_roles::UserRoles;

#[derive(Serialize, Deserialize, Hash, Debug)]
pub struct UserDTO {
    pub name: String,
    pub age: i32, /*
        postgres actually doesnt support i8 which would be closer to a reasonable age but because
        its signed we need to deal with negatives anyway so may as well make this i32 and then do
        validations
    */
    pub last_name: String,
    pub role: UserRoles
}

impl Clone for UserDTO {
    fn clone(&self) -> Self {
        UserDTO {
            name: self.name.clone(),
            age: self.age,
            last_name: self.last_name.clone(),
            role: self.role.clone()
        }
    }
}