import { AppShell, Container } from '@mantine/core';
import { Auth } from '../components/Auth/Auth';
import { Welcome } from '../components/Welcome/Welcome';

export function HomePage() {
  return (
    <AppShell padding="md">
      <AppShell.Main bg="gray.0">
        <Welcome />
        <Container size="sm">
          <Auth />
        </Container>
      </AppShell.Main>
    </AppShell>
  );
}
