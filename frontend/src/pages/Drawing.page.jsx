import { Text, Group, Container, AppShell } from "@mantine/core";
import { useNavigate, useParams } from "react-router-dom";
import Header from "../components/Header/Header";
import EditableName from "../components/EditableName";
import { useEffect, useState } from "react";
import * as api from "../api.js";
import Canvas from "../components/Canvas/index.jsx";

export function DrawingPage() {
  const { drawingId } = useParams();
  const navigate = useNavigate();
  const [drawing, setDrawing] = useState();

  useEffect(() => {
    (async () => {
      try {
        setDrawing(await api.getDrawing(drawingId));
      } catch (e) {
        if (e.message.includes("invalid auth token") || e.message.includes("auth header missing")) {
          navigate('/');
        }
      }
    })()
  }, []);

  const setDrawingName = (name) => {
    if (drawing) {
      setDrawing({ ...drawing, name });
    }
  };

  return <AppShell
      header={{ height: 60 }}
      padding="md">
    <Header size="md" leftSide={
      <Group gap="0.5rem">
        <Text size="30px" c='rgba(0,0,0,0.3)' fw={300}>/</Text>
        <EditableName value={drawing ? drawing.name : ""} onChange={setDrawingName}/>
      </Group>
    }/>
    <AppShell.Main bg="gray.0">
      {drawing && <Container size="md">
        <Canvas id={drawing.id} width={drawing.width} height={drawing.height} />
      </Container>}
    </AppShell.Main>
  </AppShell>
}
