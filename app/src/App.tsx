import { useState } from "react";
import "./App.css";
import { open } from "@tauri-apps/api/dialog";
import L from "leaflet";
import "leaflet/dist/leaflet.css";

import { MapContainer } from "react-leaflet";
import { AnvilLayer } from "./AnvilLayer";
import { Ribbon } from "./Ribbon";
import { savesDir } from "./openWorld";

function App() {
  const [worldDir, setWorldDir] = useState<string | undefined>();

  async function handleOpen() {
    const saves = await savesDir();

    const selected = await open({
      multiple: false,
      directory: true,
      defaultPath: saves ?? undefined,
    });

    if (selected && !Array.isArray(selected)) {
      setWorldDir(selected);
    }
  }

  // TODO: Actually get the world path and whatnot.
  // TODO: Some way for super zoom out? Currently causes way too many tile
  // requests.
  // TODO: Zoom in is blurry. Feels like it shouldn't be on MacOS's webview. But
  // it is.
  // TODO: Just list saves in drop down or something rather than file dialog
  // TODO: Open arb directory for world.
  // TODO: Show coordinates.
  return (
    <div className="container">
      <Ribbon>
        <button onClick={handleOpen}>Open save...</button>
      </Ribbon>
      <MapContainer
        crs={L.CRS.Simple}
        minZoom={2}
        className="map-container"
        center={[0, 0]}
        zoom={6}
        scrollWheelZoom={true}
      >
        {worldDir && <AnvilLayer worldDir={worldDir} />}
      </MapContainer>
    </div>
  );
}

export default App;
