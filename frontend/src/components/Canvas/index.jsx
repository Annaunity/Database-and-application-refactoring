import { Stack } from "@mantine/core";
import { useEffect, useRef, useState } from "react";

export default function Canvas({ width, height }) {
  const canvasRef = useRef(null);
  const ctxRef = useRef(null);
  const [devicePixelRatio, setDevicePixelRatio] = useState(window.devicePixelRatio);

  const [isDrawing, setIsDrawing] = useState(false);

  const checkZoom = () => {
    if (window.devicePixelRatio != devicePixelRatio) {
      setDevicePixelRatio(window.devicePixelRatio);
    }
  }

  const clear = () => {
    if (ctxRef.current == null) return;
    const ctx = ctxRef.current;

    ctx.fillStyle = 'white';
    ctx.fillRect(0, 0, width, height);
  };

  const onMouseDown = () => {
    setIsDrawing(true);
  };

  const onMouseUp = () => {
    setIsDrawing(false);
  };

  const onMouseMove = (ev) => {
    checkZoom();
    
    if (ctxRef.current == null) return;
    const ctx = ctxRef.current;

    const rect = canvasRef.current.getBoundingClientRect();
    const x = (ev.clientX - rect.left) * window.devicePixelRatio;
    const y = (ev.clientY - rect.top) * window.devicePixelRatio;

    if (isDrawing) {
      ctx.lineTo(x, y);
      ctx.strokeStyle = 'black';
      ctx.lineWidth = 2;
      ctx.stroke();
    } else {
      ctx.moveTo(x, y);
    }
  };

  useEffect(() => {
    const canvas = canvasRef.current;
    const ctx = canvas.getContext('2d');
    ctxRef.current = ctx;

    canvas.width = width;
    canvas.height = height;

    canvas.style.width = `${width / devicePixelRatio}px`;
    canvas.style.height = `${height / devicePixelRatio}px`;

    clear();
  }, [devicePixelRatio]);

  return <>
    <Stack>
      <canvas
        ref={canvasRef}
        width={width}
        height={height}
        onMouseDown={onMouseDown}
        onMouseUp={onMouseUp}
        onMouseMove={onMouseMove}/>
    </Stack>
  </>
}
