import { Group, Button, NumberInput, Stack, TextInput } from "@mantine/core";
import { isInRange, isNotEmpty, useForm } from "@mantine/form";
import { createDrawing } from "../../api";

export default function CreateDrawing({ onCreated }) {
  const form = useForm({
    mode: 'uncontrolled',
    initialValues: {
      name: "",
      width: 1,
      height: 1
    },

    validate: {
      name: isNotEmpty('Enter a name'),
      width: isInRange({min: 1, max: 1024}, "Enter a number between 1 and 1024"),
      height: isInRange({min: 1, max: 1024}, "Enter a number between 1 and 1024"),
    },
  });

  const submit = async (data) => {
    let drawing = await createDrawing({
      name: data.name,
      width: data.width,
      height: data.height
    });

    onCreated(drawing.id);
  };

  return (
    <form onSubmit={form.onSubmit(submit)}>
      <Stack>
        <TextInput
          withAsterisk
          label="Name"
          placeholder="Awesome drawing"
          key={form.key('name')}
          {...form.getInputProps('name')}
        />

        <Group grow>
          <NumberInput
            withAsterisk
            label="Width"
            suffix="px"
            allowNegative={false}
            allowDecimal={false}
            min={1}
            max={1024}
            key={form.key('width')}
            {...form.getInputProps('width')}
          />

          <NumberInput
            withAsterisk
            label="Height"
            suffix="px"
            allowNegative={false}
            allowDecimal={false}
            min={1}
            max={1024}
            key={form.key('height')}
            {...form.getInputProps('height')}
          />
        </Group>

        <Group justify="flex-end" mt="md">
          <Button type="submit">Create</Button>
        </Group>
      </Stack>
    </form>
  );
}
