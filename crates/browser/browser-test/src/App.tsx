import { useEffect, useState } from 'react';
import ky from 'ky';
import * as hs from './wasm'
import hs_init from './wasm'


function App() {
  
  
  const [bootstrapAddrs, setBootstrapAddrs] = useState([] as string[]);

  useEffect(() => {
    
    
    const fetchBootstrapAddrs = async () => {
      
      await hs_init()
            
      const addrs: string[] = await ky.get('http://localhost:3000/bootstrap').json();
      setBootstrapAddrs(addrs);
      console.log("addrs:", addrs);
      
      const config = new hs.Config(addrs)
      
      await hs.initialize(config)
      
      await hs.main()

    };
    

    fetchBootstrapAddrs();
  }, []); // Empty dependency array means this effect runs once on mount

  console.log(bootstrapAddrs);

  return (
    <>
      <h1>Bootstrap Addresses</h1>
      <ul>
        {bootstrapAddrs ? bootstrapAddrs.map((addr: string) => <li key={addr}>{addr}</li>) : <li>Loading...</li>}
      </ul>
    </>
  );
}

export default App;
