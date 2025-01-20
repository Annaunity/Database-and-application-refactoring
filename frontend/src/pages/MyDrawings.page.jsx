import { Text, Modal, Button, AppShell, Container, Card, Grid, Group, Title } from '@mantine/core';
import Header from "../components/Header/Header";
import { useEffect, useState } from 'react';
import { getOwnedDrawings } from '../api';
import CreateDrawing from '../components/CreateDrawing';
import { useDisclosure } from '@mantine/hooks';
import { Link } from 'react-router-dom';

export function MyDrawingsPage() {
  const [drawings, setDrawings] = useState([]);
  const [createDrawingOpened, { open: openCreateDrawing, close: closeCreateDrawing }] = useDisclosure(false);
  
  useEffect(() => {
    (async () => {
      setDrawings((await getOwnedDrawings()).items);
    })();
  }, []);

  const onCreated = () => {
    (async () => {
      setDrawings((await getOwnedDrawings()).items);
      closeCreateDrawing();
    })();
  }

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
              <Grid.Col span="auto">
                <Card key={i} shadow="xs" padding="lg" radius="md" miw="300px" maw="400px" withBorder component={Link} to={`/drawings/${drawing.id}`}>
                  <Card.Section withBorder inheritPadding py="xs">
                    <Group justify='space-between'>
                      <Text fw={500}>{drawing.name}</Text>
                      <Text c='gray'>{formatDate(drawing.updatedAt)}</Text>
                    </Group>
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
