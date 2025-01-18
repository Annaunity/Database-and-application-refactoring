import { useState } from 'react';
import { IconCat, IconDog, IconMoodConfuzed } from '@tabler/icons-react';
import {
  Anchor,
  Box,
  Button,
  Center,
  Checkbox,
  Group,
  Input,
  LoadingOverlay,
  Paper,
  PasswordInput,
  rem,
  SegmentedControl,
  Stack,
  Tabs,
  Text,
  TextInput,
} from '@mantine/core';
import { matches, isEmail, isNotEmpty, matchesField, useForm } from '@mantine/form';
import { useDisclosure } from '@mantine/hooks';
import * as api from '../../api';

export function Auth({ afterAuth }) {
  const [activeTab, setActiveTab] = useState('signIn');
  const [loadingOverlayVisible, loadingOverlay] = useDisclosure(false);

  const signInForm = useForm({
    mode: 'uncontrolled',
    initialValues: {
      usernameOrEmail: '',
      password: '',
      rememberMe: false,
    },

    validate: {
      usernameOrEmail: isNotEmpty('Enter your username or email'),
      password: isNotEmpty('Enter your password'),
    },
  });

  const signUpForm = useForm({
    mode: 'uncontrolled',
    initialValues: {
      username: '',
      email: '',
      password: '',
      confirmPassword: '',
      cutie: false,
      favouriteAnimal: 'cat',
    },

    validate: {
      username: matches(/[a-zA-Z0-9_]{4,12}/, 'Username must contain from 4 to 12 allowed characters (a-z, A-Z, 0-9, _)'),
      email: isEmail('Invalid email'),
      password: isNotEmpty('Enter a password'),
      confirmPassword: matchesField('password', 'Passwords are not the same'),
      cutie: (value) => (value ? null : 'This box must be checked'),
    },
  });

  const onSignIn = async (data) => {
    loadingOverlay.open();

    try {
      let res = await api.auth({
        usernameOrEmail: data.usernameOrEmail,
        password: data.password,
        extendSession: data.rememberMe,
      });

      window.localStorage.setItem("token", res.token);
      afterAuth();
    } catch (e) {
      if (e.message.includes("user not found")) {
        signInForm.setFieldError('usernameOrEmail', 'User not found');
      }

      if (e.message.includes("invalid credentials")) {
        signInForm.setFieldError('password', 'Invalid password');
      }

      console.log(e);
    }

    loadingOverlay.close();
  };

  const onSignUp = async (data) => {
    loadingOverlay.open();

    try {
      await api.createUser({
        username: data.username,
        email: data.email,
        password: data.password,
        favouriteAnimal: data.favouriteAnimal,
      });

      let res = await api.auth({
        usernameOrEmail: data.username,
        password: data.password,
        extendSession: false,
      });

      window.localStorage.setItem("token", res.token);
      afterAuth();
    } catch (e) {
      if (e.message.includes("username") && e.message.includes("already taken")) {
        signUpForm.setFieldError('username', 'This username is already taken');
      }

      if (e.message.includes("email") && e.message.includes("already taken")) {
        signUpForm.setFieldError('email', 'This email is already taken');
      }

      console.log(e);
    }

    loadingOverlay.close();
  };

  return (
    <Paper pos="relative" shadow="sm" radius="md" p="xl">
      <LoadingOverlay visible={loadingOverlayVisible} />

      <Tabs value={activeTab} onChange={setActiveTab} variant="pills">
        <Tabs.List justify="center">
          <Tabs.Tab value="signIn">Sign in</Tabs.Tab>
          <Tabs.Tab value="signUp">Sign up</Tabs.Tab>
        </Tabs.List>

        <Tabs.Panel value="signIn" mt="sm">
          <form onSubmit={signInForm.onSubmit(onSignIn)}>
            <Stack>
              <TextInput
                withAsterisk
                label="Username or email"
                placeholder="username or your@email.com"
                key={signInForm.key('usernameOrEmail')}
                {...signInForm.getInputProps('usernameOrEmail')}
              />

              <PasswordInput
                withAsterisk
                label="Password"
                placeholder="yourPassword123"
                key={signInForm.key('password')}
                {...signInForm.getInputProps('password')}
              />

              <Checkbox
                mt="md"
                label="Remember me"
                key={signInForm.key('rememberMe')}
                {...signInForm.getInputProps('rememberMe', { type: 'checkbox' })}
              />

              <Group justify="space-between" mt="md">
                <Anchor onClick={() => setActiveTab('signUp')}>I don't have an account</Anchor>
                <Button type="submit">Sign in</Button>
              </Group>
            </Stack>
          </form>
        </Tabs.Panel>

        <Tabs.Panel value="signUp" mt="sm">
          <form onSubmit={signUpForm.onSubmit(onSignUp)}>
            <Stack>
              <TextInput
                withAsterisk
                label="Username"
                placeholder="username"
                key={signUpForm.key('username')}
                {...signUpForm.getInputProps('username')}
              />

              <TextInput
                withAsterisk
                label="Email"
                placeholder="your@email.com"
                key={signUpForm.key('email')}
                {...signUpForm.getInputProps('email')}
              />

              <PasswordInput
                withAsterisk
                label="Password"
                placeholder="yourPassword123"
                key={signUpForm.key('password')}
                {...signUpForm.getInputProps('password')}
              />

              <PasswordInput
                withAsterisk
                label="Confirm password"
                placeholder="yourPassword123"
                key={signUpForm.key('confirmPassword')}
                {...signUpForm.getInputProps('confirmPassword')}
              />

              <Input.Wrapper label="Favourite animal" withAsterisk>
                <Box>
                  <SegmentedControl
                    value={signUpForm.getValues().favouriteAnimal}
                    onChange={v => signUpForm.setFieldValue('favouriteAnimal', v)}
                    data={[
                      {
                        value: 'cat',
                        label: (
                          <Center style={{ gap: rem(5) }}>
                            <IconCat style={{ width: rem(20), height: rem(20) }} />
                            <Text>Cat</Text>
                          </Center>
                        ),
                      },
                      {
                        value: 'dog',
                        label: (
                          <Center style={{ gap: rem(5) }}>
                            <IconDog style={{ width: rem(20), height: rem(20) }} />
                            <Text>Dog</Text>
                          </Center>
                        ),
                      },
                      {
                        value: 'unsure',
                        label: (
                          <Center style={{ gap: rem(5) }}>
                            <IconMoodConfuzed style={{ width: rem(20), height: rem(20) }} />
                            <Text>Unsure</Text>
                          </Center>
                        ),
                      },
                    ]}
                  />
                </Box>
              </Input.Wrapper>

              <Checkbox
                mt="md"
                label="By checking this box you agree that you are a cutie"
                key={signUpForm.key('cutie')}
                {...signUpForm.getInputProps('cutie', { type: 'checkbox' })}
              />

              <Group justify="space-between" mt="md">
                <Anchor onClick={() => setActiveTab('signIn')}>I already have an account</Anchor>
                <Button type="submit">Sign up</Button>
              </Group>
            </Stack>
          </form>
        </Tabs.Panel>
      </Tabs>
    </Paper>
  );
}
