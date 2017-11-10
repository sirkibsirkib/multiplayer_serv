
use std::collections::HashSet;
use serde::{Serialize,Deserialize};
use super::{ClientID,BoundedString,UserBaseError};
use ::identity::ClientIDSet;
use super::super::identity::{EntityID,LocationID};
use super::super::engine::game_state::{Point,LocationPrimitive};

//change applied to a SINGLE location
#[derive(Clone,Copy,Serialize,Deserialize,Debug)]
pub enum Diff {
    MoveEntityTo(EntityID,Point),
    PlaceInside(EntityID,Point),
}

//PRIMITIVE
#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum MsgToServer {
    CreateEntity(EntityID,Point),
    ControlMoveTo(LocationID,EntityID,Point),
    ClientHasDisconnected,
    ClientLogin(BoundedString,BoundedString),
    RequestEntityData(EntityID),
    RequestControlling,
    RequestLocationData(LocationID),
}

//PRIMITIVE
#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum MsgToClient {
    ApplyLocationDiff(LocationID,Diff),
    GiveControlling(EntityID,LocationID),
    GiveLocationPrimitive(LocationID,LocationPrimitive),
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
// #[derive(Serialize, Deserialize, Copy, Clone, Debug)]
// #[derive(Copy,Clone,Debug)]
pub enum MsgToClientSet {
    Only(MsgToClient, ClientID),
    All(MsgToClient),
    Subset(MsgToClient,ClientIDSet),
}
