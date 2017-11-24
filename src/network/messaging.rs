
use super::{BoundedString,UserBaseError};
use ::identity::*;
use ::points::*;
use ::engine::game_state::locations::LocationPrimitive;
use ::engine::entities::{EntityData};
use ::engine::objects::{ObjectData};
use ::engine::game_state::worlds::WorldPrimitive;

//change applied to a SINGLE location
#[derive(Clone,Copy,Serialize,Deserialize,Debug)]
pub enum Diff {
    MoveEntityTo(EntityID,DPoint2),
    PlaceInside(EntityID,DPoint2),
}

//PRIMITIVE
#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum MsgToServer {
    CreateEntity(EntityID,DPoint2),
    ControlMoveTo(LocationID,EntityID,DPoint2),
    ClientHasDisconnected,
    ClientLogin(BoundedString,BoundedString),
    RequestEntityData(EntityID),
    RequestObjectData(ObjectID),
    RequestControlling,
    RequestLocationData(LocationID),
    RequestWorldData(WorldID),
}

//PRIMITIVE
#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum MsgToClient {
    GiveEntityData(EntityID,EntityData),
    GiveObjectData(ObjectID,ObjectData),
    ApplyLocationDiff(LocationID,Diff),
    GiveControlling(EntityID,LocationID),
    GiveLocationPrimitive(LocationID,LocationPrimitive),
    GiveWorldPrimitive(WorldID,WorldPrimitive),
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
pub enum MsgToClientSet {
    Only(MsgToClient, ClientID),
    All(MsgToClient),
    Subset(MsgToClient,ClientIDSet),
}
