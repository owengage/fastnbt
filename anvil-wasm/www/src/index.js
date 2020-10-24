class WorkerPool {
    constructor(count, onMessage) {
        this.workers = [];
        this.messageCount = 0;
        for (let i = 0; i < count; i++) {
            const worker = new Worker('./worker.bundle.js');
            worker.onmessage = onMessage;
            worker.onerror = console.error;
            this.workers.push(worker);
        }
    }

    postMessage(data) {
        const i = this.messageCount % this.workers.length;
        this.messageCount++;

        this.workers[i].postMessage(data);
    }
}

const workers = new WorkerPool(navigator.hardwareConcurrency, e => {
    const { fileName, data } = e.data
    if (e.data.fileName) {
        console.log('rendered:', fileName);

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

const tileCache = {};

let files = [];
const callbacks = {};


var CanvasLayer = L.GridLayer.extend({
    createTile: function (coords, done) {
        // in minecraft x/z is the floor, but in leaflet x/y is.
        const fileName = `r.${coords.x}.${coords.y}.mca`

        // Check the cache for the tile first.
        const cached = tileCache[fileName];
        if (cached) {
            console.log("cached:", fileName);
            return cached.tile
        }

        const file = files.find(file => file.name == fileName);

        var tile = L.DomUtil.create('canvas', 'leaflet-tile');
        var size = this.getTileSize();
        tile.width = size.x;
        tile.height = size.y;

        const reader = new FileReader();
        reader.onload = ev => {
            const region = ev.target.result;

            console.log("rendering:", fileName);
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

const inputElement = document.getElementById("region_files");
inputElement.addEventListener("change", handleFiles, false);
function handleFiles() {
    files = Array.from(this.files);
    mymap.eachLayer(layer => layer.redraw());
};

var mymap = L.map("mapid", {
    crs: L.CRS.Simple
}).setView([0, 0], 1);

mymap.addLayer(new CanvasLayer({
    minNativeZoom: 1,
    maxNativeZoom: 1,
    tileSize: 512,
    noWrap: true,
}));