use super::game_state::EntityID;

struct PlayerData {
    username : String,
    hashed_password : String,
    owns : Vec<EntityID>,

    //entities that enter the world with the next login (consuming them)
    stored_entities : Vec<EntityID>
}
