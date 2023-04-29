import { useEffect } from 'react';
import L from 'leaflet';
import { useLeafletContext } from "@react-leaflet/core";
import { UnlistenFn } from '@tauri-apps/api/event';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/tauri';
import { WorldInfo } from './WorldSelect';

type HeightmapMode = "trust" | "calculate";

interface AnvilLayerProps {
    /// Base path of a Minecraft world. This should be the directory containing
    /// the entire world, so the DIM1, DIM-1 and region directories are within
    /// this directory.
    world: WorldInfo,
    dimension: string,
    heightmapMode: HeightmapMode,
}

export function AnvilLayer({ world, heightmapMode, dimension }: AnvilLayerProps) {
    const context = useLeafletContext();

    useEffect(() => {
        const container = context.layerContainer || context.map;
        const layer = make_layer({ worldDir: world.dir, heightmapMode, dimension }, {
            minNativeZoom: 6,
            maxNativeZoom: 6,
            tileSize: 512,
            noWrap: true,
            keepBuffer: 10,
        });
        let unlisten: UnlistenFn | null = null;
        let destructed = false;

        listen<TileResponse>('tile_rendered', (event) => {
            layer.handleTileResponse(event.payload)
        }).then((unlistenFn) => {
            // Handle the race condition where the component is unmounted before
            // this runs. If the destructor has already ran, it won't call
            // unlisten, in which case we call it here.
            if (destructed) {
                unlistenFn();
            }
            // if the destructor hasn't run, we save this for it to call.
            unlisten = unlistenFn;
        })

        container.addLayer(layer);

        return () => {
            destructed = true;
            container.removeLayer(layer);
            unlisten && unlisten();
        };
    }, [world.dir, heightmapMode, dimension]);

    return null;
}

function make_layer(args: AnvilLayerInnerArgs, leafletOpts: L.GridLayerOptions): AnvilLayerInner & L.Layer {
    // Leaflet types don't figure out the constructor args. These are the same
    // as get passed to the initialize function in the extend function below.
    // This makes a TS safe wrapper around the problem.

    // @ts-ignore
    return new _AnvilLayerInner({ ...args, ...leafletOpts });
}

interface AnvilLayerInnerArgs {
    worldDir: string,
    dimension: string,
    heightmapMode: HeightmapMode,
}

interface AnvilLayerInner {
    handleTileResponse: (resp: TileResponse) => void;
}

type TileResponse = TileRender | TileMissing | TileError;

interface TileRender {
    kind: "render",
    id: number,
    rx: number,
    rz: number,
    dimension: string,
    imageData: string,
}

interface TileError {
    kind: "error",
    id: number,
    rx: number,
    rz: number,
    dimension: string,
    message: string,
}

interface TileMissing {
    kind: "missing",
    id: number,
    rx: number,
    rz: number,
    dimension: string,
}

interface Callback {
    done: (error: Error | null, tile: HTMLImageElement) => void,
    tile: HTMLImageElement,
}

type CallbackMap = Map<string, Callback>;

const _AnvilLayerInner = L.GridLayer.extend({
    initialize: function (args: AnvilLayerInnerArgs) {
        // @ts-ignore
        L.GridLayer.prototype.initialize.call(this, args);

        this.id = awfulRandomNumber();
        this.args = args;
        this.callbacks = new Map();


        this.handleTileResponse = (resp: TileResponse) => {

            const callbacks: CallbackMap = this.callbacks;
            const val = callbacks.get(`${resp.rx},${resp.rz}`);

            if (resp.id !== this.id) {
                // This is from a different map. This can occur if we send a
                // request for a tile, but only receive it after some
                // options were changed.
                return;
            }

            if (!val) {
                console.error("Tile response for unrequested tile.");
                return;
            }
            let { done, tile } = val;

            switch (resp.kind) {
                case "render": {
                    tile.src = "data:image/png;base64," + resp.imageData;
                    const key = `${resp.rx},${resp.rz}`;
                    callbacks.set(key, { done, tile });
                    break;
                }
                case "missing":
                    break;
                case "error":
                    console.error(resp);
                    break;
                default:
                    console.error("Unexpected callback received");
                    break;
            };
            done(null, tile);
        };
    },
    createTile: function (coords: any, done: any) {
        const args: AnvilLayerInnerArgs = this.args;
        const callbacks: CallbackMap = this.callbacks;

        // in minecraft x/z is the floor, but in leaflet x/y is.
        const req = {
            id: this.id,
            tile: {
                rx: coords.x,
                rz: coords.y,
                dimension: args.dimension,
                worldDir: args.worldDir,
                heightmapMode: args.heightmapMode,
            },
        };

        const key = `${req.tile.rx},${req.tile.rz}`;

        var tile = L.DomUtil.create('img', 'leaflet-tile');
        var size = this.getTileSize();
        tile.width = size.x;
        tile.height = size.y;

        callbacks.set(key, { done, tile });
        invoke('render_tile', req);
        return tile;
    }
});


async function fromB64(data: string): Promise<Uint8ClampedArray> {
    const dataUrl = "data:application/octet-binary;base64," + data;

    // Use fetch to convert the base64.
    const resp = await fetch(dataUrl);
    const buf = await resp.arrayBuffer();
    return new Uint8ClampedArray(buf);
}

function awfulRandomNumber(): number {
    // don't judge me.
    return Math.ceil(Math.random() * 1e6)
}