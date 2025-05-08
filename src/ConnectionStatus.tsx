import { HStack, Icon, Text } from "@chakra-ui/react";
import { VscCircleFilled } from "react-icons/vsc";

type ConnectionStatusProps = {
  connection: "connected" | "disconnected" | "desynchronized";
  darkMode: boolean;
  isCollapsed?: boolean;
};

function ConnectionStatus({ connection, darkMode, isCollapsed = false }: ConnectionStatusProps) {
  const statusColor = {
    connected: "green.500",
    disconnected: "orange.500",
    desynchronized: "red.500",
  }[connection];

  const statusText = {
    connected: "You are connected!",
    disconnected: "Connecting to the server...",
    desynchronized: "Disconnected, please refresh.",
  }[connection];

  if (isCollapsed) {
    return (
      <Icon
        as={VscCircleFilled}
        color={statusColor}
        boxSize={4}
        data-connection-status="true"
      />
    );
  }

  return (
    <HStack spacing={1}>
      <Icon
        as={VscCircleFilled}
        color={statusColor}
        data-connection-status="true"
      />
      <Text
        fontSize="sm"
        fontStyle="italic"
        color={darkMode ? "gray.300" : "gray.600"}
      >
        {statusText}
      </Text>
    </HStack>
  );
}

export default ConnectionStatus;
