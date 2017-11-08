
use serde::{Serialize,Deserialize};
use super::{ClientID,BoundedString,UserBaseError};
use super::super::identity::{EntityID,LocationID};
use super::super::engine::game_state::{Point};

//PRIMITIVE
#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum MsgToServer {
    RequestControlOf(EntityID),
    RelinquishControlof(EntityID),
    CreateEntity(EntityID,Point),
    ControlMoveTo(LocationID,EntityID,Point),
    //username, password_hash
    ClientLogin(BoundedString,BoundedString),
    RequestEntityData(EntityID),
    RequestControlling,
    RequestLocationData(LocationID),
}

//PRIMITIVE
#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum MsgToClient {
    GiveEntityData(EntityID,LocationID,Point),
    GiveControlling(Option<(EntityID,LocationID)>),
    CreateEntity(EntityID,Point),
    YouNowControl(EntityID),
    YouNoLongerControl(EntityID),
    EntityMoveTo(EntityID,Point),
    LoginSuccessful(ClientID),
    LoginFailure(UserBaseError),
}


//WRAPS MsgToServer
#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct MsgFromClient {
    pub msg : MsgToServer,
    pub cid : ClientID,
}

//WRAPS MsgToClient
#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum MsgToClientSet {
    Only(MsgToClient, ClientID),
    All(MsgToClient),
}
