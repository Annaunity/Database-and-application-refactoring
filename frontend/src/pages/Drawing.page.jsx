import { NumberInput, Card, Stack, Paper, Modal, Text, Group, Container, AppShell, Title, Button } from "@mantine/core";
import { useNavigate, useParams } from "react-router-dom";
import Header from "../components/Header/Header";
import EditableName from "../components/EditableName";
import { useEffect, useState } from "react";
import * as api from "../api.js";
import Canvas from "../components/Canvas/index.jsx";
import { useDisclosure } from "@mantine/hooks";
import { IconHistory, IconResize, IconTrash } from "@tabler/icons-react";

export function DrawingPage() {
  const { drawingId } = useParams();
  const navigate = useNavigate();
  const [drawing, setDrawing] = useState();
  const [versions, setVersions] = useState();
  const [newWidth, setNewWidth] = useState();
  const [newHeight, setNewHeight] = useState();
  const [versionsOpen, { open: openVersions, close: closeVersions }] = useDisclosure(false);
  const [resizeOpen, { open: openResize, close: closeResize }] = useDisclosure(false);

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

  const updateVersions = async () => {
    let versions = (await api.getDrawingVersions(drawingId)).items;
    await Promise.all(versions.map(async (version) => {
      let blob = await api.getDrawingVersionThumbnail(drawing.id, version.id);
      version.src = URL.createObjectURL(blob);
    }));
    setVersions(versions);
  };

  const setDrawingName = (name) => {
    if (drawing) {
      api.updateDrawing(drawingId, { name });
      setDrawing({ ...drawing, name });
    }
  };

  const formatDate = (date) => {
    return (new Intl.DateTimeFormat('en-US', {
      day: 'numeric',
      month: 'short',
      hour: 'numeric',
      minute: 'numeric',
    })).format(new Date(date));
  }

  const deleteDrawing = async () => {
    await api.deleteDrawing(drawingId);
    navigate('/my/drawings');
  };

  const revertToVersion = async (versionId) => {
    let blob = await api.getDrawingVersion(drawingId, versionId);
    await api.uploadDrawingNewVersion(drawingId, blob);
    closeVersions();
    window.location.reload();
  };

  const prepareOpenResize = () => {
    setNewWidth(drawing.width);
    setNewHeight(drawing.height);
    openResize();
  };

  const doResize = async () => {
    await api.updateDrawing(drawingId, { width: newWidth, height:newHeight });
    closeResize();
    window.location.reload();
  }

  return <>
    <Modal opened={versionsOpen} onClose={closeVersions} title="Versions">
      {versions && <Stack>
        {versions.map((version, i) => 
          <Card key={i} shadow="xs" padding="lg" radius="md" miw="300px" maw="400px" withBorder>
            <Card.Section withBorder inheritPadding py="xs">
              <Group justify="space-between">
                <Text c='gray'>{formatDate(version.createdAt)}</Text>
                <Button variant="subtle" onClick={() => revertToVersion(version.id)}>Rollback</Button>
              </Group>
            </Card.Section>
            <Card.Section>
              <center><img src={version.src || ''}/></center>
            </Card.Section>
          </Card>
        )}
      </Stack>}
    </Modal>

    <Modal opened={resizeOpen} onClose={closeResize} title="Resize image">
      <NumberInput
        withAsterisk
        label="Width"
        suffix="px"
        value={newWidth}
        onChange={(e) => setNewWidth(e)}
        allowNegative={false}
        allowDecimal={false}
        min={1}
        max={1024}/>
      <NumberInput
        my='xs'
        withAsterisk
        label="Height"
        suffix="px"
        value={newHeight}
        onChange={(e) => setNewHeight(e)}
        allowNegative={false}
        allowDecimal={false}
        min={1}
        max={1024}/>
      <Group justify="flex-end">
        <Button onClick={doResize}>Resize</Button>
      </Group>
    </Modal>

    <AppShell
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
          <Group mb='md'>
            <Button variant='outline' onClick={() => {updateVersions(); openVersions();}} leftSection={<IconHistory size="1rem"/>}>Version History</Button>
            <Button variant='outline' onClick={prepareOpenResize} color='pink' leftSection={<IconResize size="1rem"/>}>Resize drawing</Button>
            <Button variant='outline' onClick={deleteDrawing} color='red' leftSection={<IconTrash size="1rem"/>}>Delete drawing</Button>
          </Group>
          <Paper withBorder p='sm' bg='rgba(0, 0, 0, 0.05)'>
            <Canvas id={drawing.id} width={drawing.width} height={drawing.height} />
          </Paper>
        </Container>}
      </AppShell.Main>
    </AppShell>
  </>
}
