

    // MsgToServer::LoadEntities => {
    //     for e in global_state.entity_iterator() {
    //         outgoing_updates.push(
    //             MsgToClientSet::Only(MsgToClient::CreateEntity(*e.0,*e.1.p()), d.cid)
    //         );
    //     }
    // }
    // MsgToServer::RequestControlOf(eid) => {
    //
    //     if global_state.entity_exists(eid) {
    //         player_controlling.insert(d.cid, eid);
    //         outgoing_updates.push(
    //             MsgToClientSet::Only(MsgToClient::YouNowControl(eid), d.cid)
    //         );
    //     }
    //     // try_add_control(d.cid, eid, global_state, player_controlling);
    //
    // },
    // MsgToServer::CreateEntity(eid,pt) => {
    //     global_state.add_entity(eid, Entity::new(pt));
    //     outgoing_updates.push(
    //         MsgToClientSet::All(MsgToClient::CreateEntity(eid,pt))
    //     );
    // },
    // MsgToServer::ControlMoveTo(eid,pt) => {
    //     if player_controlling.get(&d.cid) == Some(&eid) {
    //         global_state.entity_move_to(eid, pt);
    //         outgoing_updates.push(
    //             MsgToClientSet::All(MsgToClient::EntityMoveTo(eid, pt))
    //         );
    //     }
    //     // if client_controls(d.cid, eid, player_controlling) {
    //     //
    //     // }
    // },
