import { useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";
import { open } from "@tauri-apps/api/dialog";
// import { readDir, BaseDirectory } from "@tauri-apps/api/fs";
import { homeDir } from "@tauri-apps/api/path";
// // const homeDirPath = await homeDir();

import "leaflet/dist/leaflet.css";

// // Reads the `$APPDATA/users` directory recursively
// const entries = await readDir(".", {
//   dir: BaseDirectory.Home,
//   recursive: true,
// });

import { MapContainer, Marker, Popup, TileLayer, useMap } from "react-leaflet";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    setGreetMsg(await invoke("greet", { name }));
  }

  async function handleOpen() {
    const home = await homeDir();

    const selected = await open({
      multiple: true,
      directory: true,
      defaultPath: `${home}/Library/Application Support/minecraft/saves`,
    });

    console.log(selected);
  }

  return (
    <div className="container">
      <MapContainer
        className="map-container"
        center={[51.505, -0.09]}
        zoom={13}
        scrollWheelZoom={true}
      >
        <TileLayer
          attribution='&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
          url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
        />
        <Marker position={[51.505, -0.09]}>
          <Popup>
            A pretty CSS3 popup. <br /> Easily customizable.
          </Popup>
        </Marker>
      </MapContainer>
    </div>
  );
}

export default App;
