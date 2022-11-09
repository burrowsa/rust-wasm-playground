import { Smiley, Snake } from "wasm-playground";

if (document.getElementById('smiley-canvas') !== null) {
    Smiley.new('smiley-canvas');
}

if (document.getElementById('snake-canvas') !== null) {
    Snake.new('snake-canvas', 15, 15);
}
