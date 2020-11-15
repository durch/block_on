 Generate a blocking method for each async method in an impl block. Supports either `tokio` or `async-std` backend.
 Generated methods are suffixed with `_blocking`.

 # Example `tokio`
 ```
 use block_on::block_on;

 struct Tokio {}

 #[block_on("tokio")]
 impl Tokio {
     async fn test_async(&self) {}        
 }
 ```

 Generates the following impl block
 ```no_run
 # struct Dummy {}
 # impl Dummy {
 async fn test_async(&self) {}
         
 fn test_async_blocking(&self) {
     use tokio::runtime::Runtime;
     let mut rt = Runtime::new().unwrap();
     rt.block_on(self.test_async())
 }
 # }
 ```

 # Example `async-std`
 ```
 use block_on::block_on;

 struct AsyncStd {}

 #[block_on("async-std")]
 impl AsyncStd {
     async fn test_async(&self) {}        
 }
 ```

 Generates the following method in the same impl block
 ```no_run
 # struct Dummy {}
 # impl Dummy {
 async fn test_async(&self) {}        

 fn test_async_blocking(&self) {
       use async_std::task;
       task::block_on(self.test_async())
 }
 # }
 ```