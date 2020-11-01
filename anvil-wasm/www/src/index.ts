import leaflet from 'leaflet';
import 'leaflet/dist/leaflet.css';

import WorkerPool from './worker-pool';

const workers = new WorkerPool(navigator.hardwareConcurrency, e => {
    const { fileName, data } = e.data
    if (e.data.fileName) {
        const { tile, done } = callbacks[fileName];

        const imageData = new Uint8ClampedArray(data);
        var ctx = tile.getContext('2d');
        ctx.putImageData(new ImageData(imageData, 512, 512), 0, 0);
        tileCache[fileName] = {
            tile,
        }
        done(null, tile);
    }
});

const tileCache: any = {};

let files: File[] = [];
const callbacks: any = {};

var MinecraftLayer = leaflet.GridLayer.extend({
    createTile: function (coords: any, done: any) {
        // in minecraft x/z is the floor, but in leaflet x/y is.
        const fileName = `r.${coords.y}.${coords.x}.mca`

        // Check the cache for the tile first.
        const cached = tileCache[fileName];
        if (cached) {
            return cached.tile
        }

        const file = files.find(file => file.name == fileName);

        var tile = leaflet.DomUtil.create('canvas', 'leaflet-tile');
        var size = this.getTileSize();
        (<any>tile).width = size.x;
        (<any>tile).height = size.y;

        const reader = new FileReader();
        reader.onload = ev => {
            const region = ev.target?.result;

            workers.postMessage({
                fileName,
                region: region,
            }, [region]);
        };

        if (file) {
            callbacks[fileName] = { tile, done };
            reader.readAsArrayBuffer(file);
        }

        return tile;
    }
});

const inputElement = document.getElementById("region_files")!;
inputElement.addEventListener("change", handleFiles, false);
function handleFiles(this: any) {
    const el: any = this;

    files = Array.from(el.files);
    mymap.eachLayer(layer => (<any>layer).redraw());
};

var mymap = leaflet.map("map", {
    crs: leaflet.CRS.Simple
}).setView([0, 0], 1);

mymap.addLayer(new (<any>MinecraftLayer)({
    minNativeZoom: 1,
    maxNativeZoom: 1,
    tileSize: 512,
    noWrap: true,
}));