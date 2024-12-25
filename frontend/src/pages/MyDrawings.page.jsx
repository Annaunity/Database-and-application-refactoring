import { AppShell, Container } from '@mantine/core';
import Header from "../components/Header/Header";

export function MyDrawingsPage() {
  return (
    <AppShell
        header={{ height: 60 }}
        padding="md">
      <Header/>
      <AppShell.Main bg="gray.0">
        <Container size="sm">
          my drawings
        </Container>
      </AppShell.Main>
    </AppShell>
  );
}
