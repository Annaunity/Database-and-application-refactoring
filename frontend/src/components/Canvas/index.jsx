import { Paper, Button, Group, Stack, Text} from "@mantine/core";
import { useEffect, useRef, useState } from "react";
import { getDrawingLatestVersion, uploadDrawingNewVersion } from "../../api";
import { IconCircleFilled } from "@tabler/icons-react";

export default function Canvas({ id, width, height }) {
  const canvasRef = useRef(null);
  const ctxRef = useRef(null);
  const [devicePixelRatio, setDevicePixelRatio] = useState(window.devicePixelRatio);

  const colors = [
    ["White", "white"],
    ["Black", "black"],
    ["Red", "#e62222"],
    ["Pink", "pink"],
    ["Blue", "#228be6"],
  ];

  const [color, setColor] = useState(1);

  const sizes = [2, 5, 10, 25];
  const [size, setSize] = useState(0);

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

    if (ctxRef.current == null) return;
    const ctx = ctxRef.current;

    ctx.beginPath();
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
      ctx.strokeStyle = colors[color][1];
      ctx.lineWidth = sizes[size];
      ctx.lineCap = 'round';
      ctx.stroke();
      ctx.moveTo(x, y);
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

    (async () => {
      let blob = await getDrawingLatestVersion(id);
      let image = await createImageBitmap(blob);
      ctx.drawImage(image, 0, 0);
    })()
  }, [devicePixelRatio]);

  const save = () => {
    if (canvasRef.current == null) return;
    canvasRef.current.toBlob((blob) => {
      (async () => {
        await uploadDrawingNewVersion(id, blob);
      })();
    }, "image/png");
  };

  const doBlur = () => {
    
  };

  const doInvertColors = () => {
    
  };

  return <>
    <Stack align="center">
      <Group>
        <Paper p='sm' shadow='sm'>
          <Text ta='center'>Brush size</Text>
          <Group justify='center' gap="xs">
            {sizes.map((c, i) =>
              <Button key={i}
                onClick={() => setSize(i)}
                variant={size == i ? 'light' : 'transparent'} leftSection={<IconCircleFilled size={c}/>}>{c}</Button>
            )}
          </Group>
        </Paper>
        <Paper p='sm' shadow='sm'>
          <Text ta='center'>Brush color</Text>
          <Group justify='center' gap="xs">
            {colors.map((c, i) =>
              <Button key={i}
                onClick={() => setColor(i)}
                variant={color == i ? 'light' : 'transparent'} color='dark' leftSection={<IconCircleFilled color={c[1]}/>}>{c[0]}</Button>
            )}
          </Group>
        </Paper>
      </Group>
      <Group align="flex-start">
        <Group>
          <Paper shadow='sm'>
            <canvas
              ref={canvasRef}
              width={width}
              height={height}
              onMouseDown={onMouseDown}
              onMouseUp={onMouseUp}
              onMouseMove={onMouseMove}/>
          </Paper>
        </Group>
        <Paper p='sm' shadow='md'>
          <Text mt='md' ta='center'>Don't forget to save your drawing!!!</Text>
          <Stack mt='md' align="stretch">
            <Button onClick={save}>Save</Button>
          </Stack>
          <Text mt='md' ta='center'>Operations</Text>
          <Stack mt='md' align="stretch">
            <Button variant='light' onClick={doBlur}>Blur</Button>
            <Button variant='light' onClick={doInvertColors}>Invert colors</Button>
          </Stack>
        </Paper>
      </Group>
    </Stack>
  </>
}
