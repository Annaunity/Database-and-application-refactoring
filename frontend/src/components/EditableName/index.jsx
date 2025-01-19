import { Text, Group, Container, TextInput } from "@mantine/core";
import { IconPencilOff } from "@tabler/icons-react";
import { useState } from "react";

export default function EditableName({ value, onChange }) {
  const [editorValue, setEditorValue] = useState(value);
  const [isEditing, setIsEditing] = useState(false);

  const startEditing = () => {
    setEditorValue(value);
    setIsEditing(true);
  };

  const stopEditing = () => {
    setIsEditing(false);
    onChange(editorValue);
  };

  const cancelEditing = () => {
    setIsEditing(false);
    setEditorValue(value);
  };

  return <>
    {!isEditing && <Text size="xl" onDoubleClick={startEditing}>{value}</Text>}
    {isEditing && <TextInput size="sm" value={editorValue}
      onChange={(v) => setEditorValue(v.target.value)}
      onBlur={stopEditing}
      onKeyDown={(e) => { if(e.keyCode == 13) { stopEditing() }}}
      rightSection={<IconPencilOff size="20px" onClick={cancelEditing}/>}/>}
  </>
}
