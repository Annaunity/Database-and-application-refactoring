import { Text, Modal, Button, AppShell, Container, Card, Grid, Group, Title } from '@mantine/core';
import Header from "../components/Header/Header";
import { useEffect, useState } from 'react';
import { getDrawingLatestVersionThumbnail, getOwnedDrawings } from '../api';
import CreateDrawing from '../components/CreateDrawing';
import { useDisclosure } from '@mantine/hooks';
import { Link } from 'react-router-dom';

export function MyDrawingsPage() {
  const [drawings, setDrawings] = useState([]);
  const [createDrawingOpened, { open: openCreateDrawing, close: closeCreateDrawing }] = useDisclosure(false);

  const loadDrawings = async () => {
    let res = (await getOwnedDrawings()).items;

    for (let drawing of res) {
      let blob = await getDrawingLatestVersionThumbnail(drawing.id);
      drawing.src = URL.createObjectURL(blob);
    }

    setDrawings(res);
  };
  
  useEffect(() => {
    loadDrawings();
  }, []);

  const onCreated = () => {
    loadDrawings();
    closeCreateDrawing();
  };

  const formatDate = (date) => {
    return (new Intl.DateTimeFormat('en-US', {
      day: 'numeric',
      month: 'short',
      hour: 'numeric',
      minute: 'numeric',
    })).format(new Date(date));
  }
  
  return <>
    <Modal opened={createDrawingOpened} onClose={closeCreateDrawing} title="Create a new drawing">
      <CreateDrawing onCreated={onCreated}/>
    </Modal>
    <AppShell
        header={{ height: 60 }}
        padding="md">
      <Header size="md"/>
      <AppShell.Main bg="gray.0">
        <Container size="md">
          <Group justify='space-between'>
            <Title order={1} mb="sm">Drawings</Title>
            <Button onClick={openCreateDrawing} variant='outline'>Create a new drawing</Button>
          </Group>
        </Container>
        <Container size="md">
          <Grid mt="md">
            {drawings.map((drawing, i) => (
              <Grid.Col key={i} span="auto">
                <Card shadow="xs" padding="lg" radius="md" miw="300px" maw="400px" withBorder component={Link} to={`/drawings/${drawing.id}`}>
                  <Card.Section withBorder inheritPadding py="xs">
                    <Group justify='space-between'>
                      <Text fw={500}>{drawing.name}</Text>
                      <Text c='gray'>{formatDate(drawing.updatedAt)}</Text>
                    </Group>
                  </Card.Section>
                  <Card.Section>
                    <center><img src={drawing.src || ''}/></center>
                  </Card.Section>
                </Card>
              </Grid.Col>
            ))}
          </Grid>
        </Container>
      </AppShell.Main>
    </AppShell>
  </>;
}
