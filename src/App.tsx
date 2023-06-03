import { createSignal } from 'solid-js'
import './App.css'
import { Routes, Route } from '@solidjs/router'

import Gallery from './components/gallery/Gallery'
import Monitor from './components/monitors/Monitor'
import About from './components/about/About'
import SideBar from './components/sidebar/Sidebar'



function App() {
  const [count, setCount] = createSignal(0)

  return (
    <div class="App">
      <div class='App-Layout'>
        <div class='Side-Component'>
          <SideBar></SideBar>
        </div>
        <div class='Main-Component'>
          <Routes>
            <Route path="/gallery" component={Gallery} />
            <Route path="/monitor" component={Monitor} />
            <Route path="/about" component={About} />
            {/* <Route path="/monitor" component={ } /> */}
            <Route
              path="/about"
              element={<div>This site was made with Solid</div>}
            />
          </Routes>

        </div>
      </div>
    </div>
  )
}

export default App
