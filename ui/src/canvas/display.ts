import { Entity } from "../types";

class Display {
  canvas: HTMLCanvasElement;
  cx: CanvasRenderingContext2D;
  w: number;
  h: number;
  pad: number;

  bind(canvas: HTMLCanvasElement, cx: CanvasRenderingContext2D) {
    this.canvas = canvas;
    this.cx = cx;
    this.scale(canvas.parentElement);
  }

  scale(parent: HTMLElement) {
    const { canvas, cx } = this;

    const w = parent.offsetWidth;
    const h = parent.offsetHeight;

    this.w = w;
    this.h = h;
    this.pad = 10;

    canvas.style.width = w + "px";
    canvas.style.height = h + "px";

    const dpi = window.devicePixelRatio;

    canvas.width = Math.floor(w * dpi);
    canvas.height = Math.floor(h * dpi);
    cx.scale(dpi, dpi);
  }

  clear() {
    this.cx.clearRect(0, 0, this.w, this.h);
  }

  drawEntity(entity: Entity) {
    const { pos, mass } = entity;
    this.cx.fillStyle = "black";
    this.cx.ellipse(pos[0], pos[1], mass, mass, 0, 0, 2 * Math.PI);
    this.cx.fill();
  }
}

export default Display;
