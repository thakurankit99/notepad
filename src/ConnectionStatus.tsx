import { Icon } from "@chakra-ui/react";
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

  return (
    <Icon
      as={VscCircleFilled}
      color={statusColor}
      boxSize={isCollapsed ? 4 : 3}
      data-connection-status="true"
    />
  );
}

export default ConnectionStatus;
