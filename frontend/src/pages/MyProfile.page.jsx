import { Text, Image, AppShell, Container, Group, Card, Title, Stack, Button } from '@mantine/core';
import Header from "../components/Header/Header";
import { useEffect, useState } from 'react';
import { getCurrentUser, getSessions } from '../api';
import kitten from '../kitten.png';
import puppy from '../puppy.png';
import axolotl from '../axolotl.png';
import { UAParser } from 'ua-parser-js';
import * as api from '../api.js';
import { useNavigate } from 'react-router-dom';
import { IconCheck, IconClock } from '@tabler/icons-react';

export function MyProfilePage() {
  const [user, setUser] = useState();
  const [sessions, setSessions] = useState();
  const navigate = useNavigate();

  useEffect(() => {
    (async () => {
      try {
        setUser(await getCurrentUser());
        setSessions(await getSessions());
      } catch (e) {
        if (e.message.includes("invalid auth token") || e.message.includes("auth header missing")) {
          navigate('/');
        }
      }
    })();
  }, []);

  const formatDate = (date) => {
    const secondsDiff = Math.round((new Date(date) - Date.now()) / 1000);
    const unitsInSec = [60, 3600, 86400, 86400 * 7, 86400 * 30, 86400 * 365, Infinity];
    const unitStrings = ["second", "minute", "hour", "day", "week", "month", "year"];
    const unitIndex = unitsInSec.findIndex((cutoff) => cutoff > Math.abs(secondsDiff));
    const divisor = unitIndex ? unitsInSec[unitIndex - 1] : 1;
    const rtf = new Intl.RelativeTimeFormat("en-US", { numeric: "auto" });
    return rtf.format(Math.floor(secondsDiff / divisor), unitStrings[unitIndex]);
  };

  const signOut = () => {
    (async () => {
      await api.endCurrentSession();
      localStorage.removeItem('token');
      navigate('/');
    })()
  };

  const endSession = (tokenId, isCurrent) => {
    (async () => {
      await api.endSession(tokenId);
      setSessions({ items: sessions.items.filter(s => s.tokenId != tokenId) });
      if (isCurrent) {
        localStorage.removeItem('token');
        navigate('/');
      }
    })()
  };
    
  return (
    <AppShell
        header={{ height: 60 }}
        padding="md">
      <Header size="sm"/>
      <AppShell.Main bg="gray.0">
        {user && <Container size="sm">
          <Group align='center'>
            <Card radius="md" padding="xl">
              <Card.Section>
                {user.favouriteAnimal == 'cat' && <Image
                  src={kitten}
                  w={132}
                  h={132}
                  alt="Cat" radius="md"
                />}
                {user.favouriteAnimal == 'dog' && <Image
                  src={puppy}
                  w={132}
                  h={132}
                  alt="Dog" radius="md"
                />}
                {user.favouriteAnimal != 'cat' && user.favouriteAnimal != 'dog' && <Image
                  src={axolotl}
                  w={132}
                  h={132}
                  alt="Axolotl" radius="md"
                />}
              </Card.Section>
            </Card>
            <Card shadow="sm" radius="md" flex="1" withBorder>
              <Card.Section withBorder px="sm" py="sm">
                <Group justify='space-between'>
                  <Text fw={600} size="xl">{user.username}</Text>
                  <Button variant='outline' color='red' onClick={signOut}>Sign out</Button>
                </Group>
              </Card.Section>
              <Card.Section withBorder px="sm" py="sm">
                <Group justify='space-between'>
                  <Stack gap={0}>
                    <Text><b>Email:</b> {user.email}</Text>
                    <Text><b>Favourite animal:</b> {user.favouriteAnimal}</Text>
                  </Stack>
                  <Button variant='outline' color='blue'>Edit</Button>
                </Group>
              </Card.Section>
            </Card>
          </Group>
        </Container>}
        {sessions && <Container size="sm" mt="lg">
          <Title order={1} mb="sm">Sessions</Title >
          {sessions.items.map((session, i) => {
            const { browser, cpu, device } = UAParser(session.userAgent);
            let browserParts = [browser.name, browser.version].filter(x => x != null).join(' ');
            let deviceParts = [device.vendor, device.model, device.type, cpu.architecture].filter(x => x != null).join(' ');

            return <Card key={i} shadow={session.isCurrent ? "md" : "xs"} withBorder radius="md" mb="sm">
              <Card.Section withBorder px="sm" py="sm">
                <Group justify='space-between' wrap='nowrap'>
                  {session.isCurrent && <Group gap="xs"><IconCheck/><Text>Current session</Text></Group> }
                  {!session.isCurrent && <Group gap="xs"><IconClock/><Text>Last seen {formatDate(session.lastUsedAt)}</Text></Group> }
                  <Button variant='outline' color='red' flex='0 0 auto' onClick={() => endSession(session.tokenId, session.isCurrent)}>End session</Button>
                </Group>
              </Card.Section>
              <Card.Section withBorder px="sm" py="sm">
                <Text truncate='end'><b>Browser:</b> {browserParts}</Text>
                <Text truncate='end'><b>Device:</b> {deviceParts}</Text>
                <Text truncate='end'><b>User agent:</b> {session.userAgent}</Text>
                <Text truncate='end'><b>IP address:</b> {session.ipAddress}</Text>
              </Card.Section>
            </Card>
          })}
        </Container>}
      </AppShell.Main>
    </AppShell>
  );
}
