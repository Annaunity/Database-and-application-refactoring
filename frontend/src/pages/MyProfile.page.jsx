import { AppShell, Container } from '@mantine/core';
import Header from "../components/Header/Header";

export function MyProfilePage() {
  return (
    <AppShell
        header={{ height: 60 }}
        padding="md">
      <Header/>
      <AppShell.Main bg="gray.0">
        <Container size="lg">
          my profile
        </Container>
      </AppShell.Main>
    </AppShell>
  );
}
