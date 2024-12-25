import { AppShell, Container } from '@mantine/core';
import { Auth } from '../components/Auth/Auth';
import { Welcome } from '../components/Welcome/Welcome';
import { useNavigate } from 'react-router-dom';
import { useEffect } from 'react';
import * as api from '../api';

export function HomePage() {
  const navigate = useNavigate();

  useEffect(() => {
    (async () => {
      const token = window.localStorage.getItem("token");
      if (token != null) {
        try {
          let currentUser = await api.getCurrentUser();
          window.localStorage.setItem("username", currentUser.username);
          navigate("/my");
        } catch (e) {
          console.error(e);
        }
      }
    })()
  }, []);

  return (
    <AppShell padding="md">
      <AppShell.Main bg="gray.0">
        <Welcome />
        <Container size="sm">
          <Auth afterAuth={() => navigate("/my")}/>
        </Container>
      </AppShell.Main>
    </AppShell>
  );
}
