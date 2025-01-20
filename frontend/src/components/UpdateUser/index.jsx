import { Checkbox, Text, SegmentedControl, Box, Center, Input, Group, Button, Stack, TextInput, rem, PasswordInput, Paper } from "@mantine/core";
import { IconCat, IconDog, IconMoodConfuzed, } from '@tabler/icons-react';
import { isEmail, useForm } from "@mantine/form";
import { updateUser } from "../../api";

export default function UpdateUser({ user, onUpdated }) {
  const form = useForm({
    mode: 'uncontrolled',
    initialValues: {
      email: user.email,
      favouriteAnimal: user.favouriteAnimal,
      changePassword: false,
      oldPassword: '',
      password: '',
      confirmPassword: '',
    },

    validate: {
      email: isEmail('Invalid email'),
    },
  });

  const submit = async (data) => {
    if (data.changePassword && data.oldPassword.trim() == "") {
      form.setFieldError("oldPassword", "Enter old password");
      return;
    }

    if (data.changePassword && data.password.trim() == "") {
      form.setFieldError("password", "Enter new password");
      return;
    }

    if (data.changePassword && data.confirmPassword.trim() == "") {
      form.setFieldError("confirmPassword", "Enter new password confirmation");
      return;
    }

    if (data.changePassword && data.password != data.confirmPassword) {
      form.setFieldError("confirmPassword", "Passwords do not match");
      return;
    }

    let update = {
      email: data.email,
      favouriteAnimal: data.favouriteAnimal
    };

    if (data.changePassword) {
      update.updatePassword = {
        oldPassword: data.oldPassword, 
        newPassword: data.password, 
      };
    }

    let newUser = await updateUser(user.username, update);

    onUpdated(newUser);
  };

  return (
    <form onSubmit={form.onSubmit(submit)}>
      <Stack gap={0}>
        <TextInput
          mb='xs'
          withAsterisk
          label="Email"
          placeholder="your@email.com"
          key={form.key('email')}
          {...form.getInputProps('email')}
        />

        <Input.Wrapper label="Favourite animal" withAsterisk mb='xs'>
          <Box>
            <SegmentedControl
              value={form.getValues().favouriteAnimal}
              onChange={v => form.setFieldValue('favouriteAnimal', v)}
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
          my='xs'
          label="Change password"
          key={form.key('changePassword')}
          {...form.getInputProps('changePassword', { type: 'checkbox' })}
        />

        {form.getValues().changePassword && <Paper withBorder p='xs'>
          <PasswordInput
            withAsterisk
            mb='xs'
            label="Old password"
            placeholder="yourPassword123"
            key={form.key('oldPassword')}
            {...form.getInputProps('oldPassword')}
          />

          <PasswordInput
            withAsterisk
            mb='xs'
            label="New password"
            placeholder="yourNewPassword123"
            key={form.key('password')}
            {...form.getInputProps('password')}
          />

          <PasswordInput
            withAsterisk
            label="Confirm new password"
            placeholder="yourNewPassword123"
            key={form.key('confirmPassword')}
            {...form.getInputProps('confirmPassword')}
          />
        </Paper>}

        <Group justify="flex-end" mt="md">
          <Button type="submit">Save</Button>
        </Group>
      </Stack>
    </form>
  );
}
