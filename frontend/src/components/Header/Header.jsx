import { Text, Image, AppShell, Button, Group } from '@mantine/core';
import { IconFile, IconUser } from '@tabler/icons-react';
import { useLocation, useNavigate } from 'react-router-dom';
import logo from '/src/logo256x256.png';

const routes = [
  {
    icon: IconFile,
    path: "/my/drawings",
    label: "My Drawings"
  },
  {
    icon: IconUser,
    path: "/my/profile",
    label: "My Profile"
  }
];

export default function Header() {
  const navigate = useNavigate();
  const location = useLocation();

  return (
    <AppShell.Header bg="gray.0">
      <Group h="100%" px="md">
        <Group justify="space-between" style={{ flex: 1 }}>
          <Group gap="0.5rem">
            <Image w={32} h={32} src={logo} />
            <Text fw={600} size="xl">Drawing App</Text>
          </Group>
          <Group ml="xl" gap="md">
            {routes.map((route, i) => (
              <Button
                key={i}
                variant={location.pathname === route.path ? 'filled' : 'transparent'} 
                onClick={() => navigate(route.path)}
                leftSection={<route.icon size="1.5rem" stroke={1.5} />}
              >
                {route.label}
              </Button>
            ))}
          </Group>
        </Group>
      </Group>
    </AppShell.Header>
  );
}
