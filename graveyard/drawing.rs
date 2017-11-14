
// let tex = Rc::new(Texture::from_path(
//     &mut window.factory,
//     "./assets/asset_0.png",
//     Flip::None,
//     &TextureSettings::new()
// ).unwrap());
// let aid = ;



    // if let Some(ref v) = my_data.view {
    //     window.draw_2d(&e, |c, g| {
    //         clear([0.0, 0.0, 0.0, 1.0], g);
    //         for (oid, pt_set) in v.get_location().object_iterator() {
    //             if let Ok(tex) = asset_manager.get_texture_for_oid(0) {
    //                 for pt in pt_set {
    //                     let screen_pt = v.translate_pt(*pt);
    //                     if is_on_screen(&screen_pt) {
    //                         image(tex, c.transform
    //                             .trans(screen_pt[0], screen_pt[1]), g);
    //                     }
    //                 }
    //             } else {
    //                 //get aid for oid
    //             }
    //         }
    //     });
    // }

// let screen_pt = v.translate_pt(*pt);
// let mut sprite = Sprite::from_texture(tex.clone());
// sprite.set_position(screen_pt[0], screen_pt[1]);
// window.draw_2d(event, |c, g| {
//     image(sprite, c.transform, g);
// });
// window.draw_2d(event, |c, g| {
//     image(tex, c.transform, g);
// });


// let el = [
//     screen_pt[0] - rad,
//     screen_pt[1] - rad,
//     rad*2.0,
//     rad*2.0
// ];
// // println!("client sees eid {:?} ellipse {:?}", &eid, &el);
// window.draw_2d(event, |context, graphics| {
//             ellipse(
//                 col,
//                 el,
//                 context.transform,
//                 graphics
//           );
//       }
// );



// fn render_location<E>(event : &E,
//                    window : &mut PistonWindow,
//                    my_data : &mut MyData,
//                    outgoing_update_requests : &mut Vec<MsgToServer>,
//                    asset_manager : &mut AssetManager,
//                    entity_data : & EntityDataSet,
//                    entity_data_suppressed_until : &mut Timer,
//                ) where E : GenericEvent {
//
//     if let Some(ref v) = my_data.view {
//         let mut missing_eid_assets = vec![];
//         window.draw_2d(&e, |c, g| {
//             clear([0.0, 0.0, 0.0, 1.0], g);
//             for p in &mut positions {
//                 let (x, y) = *p;
//                 *p = (x + (rand::random::<f64>() - 0.5) * 0.01,
//                       y + (rand::random::<f64>() - 0.5) * 0.01);
//             }
//             for i in 0..texture_count {
//                 let p = positions[i];
//                 image(&textures[i], c.transform
//                     .trans(p.0 * 1024.0, p.1 * 1024.0).zoom(size), g);
//             }
//         });
//         for (oid, pt_set) in v.get_location().object_iterator() {
//             let o_col = [0.2, 0.2, 0.2, 1.0];
//             // let tex : &G2dTexture = asset_manager.get_texture_for(ent_data.aid); // NONSESNSE
//             for pt in pt_set.iter() {
//                 render_something_at(*pt, v, event, window, o_col);
//             }
//         }
//         for (eid, pt) in v.get_location().entity_iterator() {
//             if missing_eid_assets.contains(&eid) {
//                 //already did this dance. waiting for it to arrive
//                 continue;
//             }
//             if let Some(ent_data) = entity_data.get(*eid) {
//                 let tex : &G2dTexture = asset_manager.get_texture_for(ent_data.aid);
//                 let col = if am_controlling(*eid, &my_data) {
//                     [0.0, 1.0, 0.0, 1.0] //green
//                 } else {
//                     [0.7, 0.7, 0.7, 1.0] //gray
//                 };
//                 render_something_at(*pt, v, event, window, col);
//             } else {
//                 missing_eid_assets.push(eid);
//                 continue;
//             }
//         }
//         if ! missing_eid_assets.is_empty() {
//             let now = Instant::now();
//             if entity_data_suppressed_until.ins < now {
//                 for eid in missing_eid_assets {
//                     outgoing_update_requests.push(
//                         MsgToServer::RequestEntityData(*eid)
//                     ); // request to populate asset manager
//                 }
//                 entity_data_suppressed_until.ins = now + entity_data_suppressed_until.setdur;
//             }
//         }
//     }
// }






// let assets = find_folder::Search::ParentsThenKids(3, 3)
//     .for_folder("assets").unwrap();
// let test_path = assets.join("test.png");
// let rust_logo: G2dTexture = Texture::from_path(
//         &mut window.factory,
//         &test_path,
//         Flip::None,
//         &TextureSettings::new()
//     ).unwrap();
// this is just a local vector to batch requests. populating this essentially populates client_out






// window.draw_2d(&e, |c, g| {
//     image(&rust_logo, c.transform, g);
// });
