import { Text, Title } from '@mantine/core';
import classes from './Welcome.module.css';

export function Welcome() {
  return (
    <>
      <Title className={classes.title} ta="center" mt={100}>
        Start{' '}
        <Text inherit variant="gradient" component="span" gradient={{ from: 'blue', to: 'green' }}>
          Drawing
        </Text>
      </Title>
      <Text c="dimmed" ta="center" size="lg" maw={580} mx="auto" mt="xl" mb="xl">
        This is a "Refactoring databases and applications" by Alex and Ann.
      </Text>
    </>
  );
}
