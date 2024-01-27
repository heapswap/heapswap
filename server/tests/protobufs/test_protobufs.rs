extern crate capnp;

use heapswap::schemas::{point};

#[test]
fn capnp_serialize_test(){
  let mut builder = capnp::message::Builder::new_default();
  
  {
    // And now we can set up the actual message we're trying to create
    let mut point_msg = builder.init_root::<point::Builder>();

    // Stuff our message with some content
    point_msg.set_x(12);

    point_msg.set_y(14);
}


    // It's now time to serialize our message to binary. Let's set up a buffer for that:
    let mut buffer = Vec::new();
    
   // And actually fill that buffer with our data
   capnp::serialize::write_message(&mut buffer, &builder).unwrap();
  
  
    // Finally, let's deserialize the data
    let deserialized = capnp::serialize::read_message(
      &mut buffer.as_slice(),
      capnp::message::ReaderOptions::new()
  ).unwrap();
  
    // `deserialized` is currently a generic reader; it understands
    // the content of the message we gave it (i.e. that there are two
    // int32 values) but doesn't really know what they represent (the Point).
    // This is where we map the generic data back into our schema.
    
    // not thread safe
    //let point_reader = deserialized.get_root::<point::Reader>().unwrap();
    
    // We can now get our x and y values back, and make sure they match
    //assert_eq!(point_reader.get_x(), 12);
    //assert_eq!(point_reader.get_y(), 14); 
    
    // thread safe
    let point_reader: capnp::message::TypedReader<capnp::serialize::OwnedSegments, point::Owned> =
        capnp::message::TypedReader::new(deserialized);

    
    
    // By using `point_reader` inside the new thread, we're hoping that Rust can safely move
    // the reference and invalidate the original thread's usage. Since the original thread
    // doesn't use `point_reader` again, this should be safe, right?
    let handle = std::thread::spawn(move || {

      // The point_reader owns its data, and we use .get() to retrieve the actual point_capnp::point::Reader
      // object from it
      let point_root = point_reader.get().unwrap();

      assert_eq!(point_root.get_x(), 12);

      assert_eq!(point_root.get_y(), 14);
  });

      handle.join().unwrap();
        
    
    
    
}