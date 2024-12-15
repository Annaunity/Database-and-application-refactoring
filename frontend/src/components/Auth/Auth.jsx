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
import { isEmail, isNotEmpty, matchesField, useForm } from '@mantine/form';
import { useDisclosure } from '@mantine/hooks';

export function Auth() {
  const [activeTab, setActiveTab] = useState('signIn');
  const [loadingOverlayVisible, loadingOverlay] = useDisclosure(false);

  const signInForm = useForm({
    mode: 'uncontrolled',
    initialValues: {
      email: '',
      password: '',
      rememberMe: false,
    },

    validate: {
      email: isEmail('Invalid email'),
      password: isNotEmpty('Enter your password'),
    },
  });

  const signUpForm = useForm({
    mode: 'uncontrolled',
    initialValues: {
      email: '',
      password: '',
      confirmPassword: '',
      cutie: false,
    },

    validate: {
      email: isEmail('Invalid email'),
      password: isNotEmpty('Enter a password'),
      confirmPassword: matchesField('password', 'Passwords are not the same'),
      cutie: (value) => (value ? null : 'This box must be checked'),
    },
  });

  return (
    <Paper pos="relative" shadow="sm" radius="md" p="xl">
      <LoadingOverlay visible={loadingOverlayVisible} />

      <Tabs value={activeTab} onChange={setActiveTab} variant="pills">
        <Tabs.List justify="center">
          <Tabs.Tab value="signIn">Sign in</Tabs.Tab>
          <Tabs.Tab value="signUp">Sign up</Tabs.Tab>
        </Tabs.List>

        <Tabs.Panel value="signIn" mt="sm">
          <form onSubmit={signInForm.onSubmit((values) => console.log(values))}>
            <Stack>
              <TextInput
                withAsterisk
                label="Email"
                placeholder="your@email.com"
                key={signInForm.key('email')}
                {...signInForm.getInputProps('email')}
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

              <Group justify="flex-end" mt="md">
                <Button type="submit">Sign in</Button>
              </Group>
            </Stack>
          </form>
        </Tabs.Panel>

        <Tabs.Panel value="signUp" mt="sm">
          <form onSubmit={signUpForm.onSubmit((values) => console.log(values))}>
            <Stack>
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
                <Button type="submit">Sign in</Button>
              </Group>
            </Stack>
          </form>
        </Tabs.Panel>
      </Tabs>
    </Paper>
  );
}
