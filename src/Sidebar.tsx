import {
  Button,
  Container,
  Flex,
  Heading,
  Link,
  Select,
  Stack,
  Switch,
  Text,
  useToast,
  Box,
  Icon,
} from "@chakra-ui/react";
import { VscRepo, VscPerson } from "react-icons/vsc";

import ConnectionStatus from "./ConnectionStatus";
import languages from "./languages.json";
import type { UserInfo } from "./rustpad";

export type SidebarProps = {
  connection: "connected" | "disconnected" | "desynchronized";
  darkMode: boolean;
  language: string;
  currentUser: UserInfo;
  users: Record<number, UserInfo>;
  onDarkModeChange: () => void;
  onLanguageChange: (language: string) => void;
  onLoadSample: () => void;
  onChangeName: (name: string) => void;
  onChangeColor: () => void;
};

function Sidebar({
  connection,
  darkMode,
  language,
  currentUser,
  users,
  onDarkModeChange,
  onLanguageChange,
  onLoadSample,
  onChangeName,
  onChangeColor,
}: SidebarProps) {
  const toast = useToast();

  return (
    <Container
      w={{ base: "3xs", md: "2xs", lg: "xs" }}
      display={{ base: "none", sm: "block" }}
      bgColor={darkMode ? "#252526" : "#f3f3f3"}
      overflowY="auto"
      maxW="full"
      lineHeight={1.4}
      py={4}
    >
      <ConnectionStatus darkMode={darkMode} connection={connection} />

      <Flex justifyContent="space-between" mt={4} mb={1.5} w="full">
        <Heading size="sm">Dark Mode</Heading>
        <Switch isChecked={darkMode} onChange={onDarkModeChange} />
      </Flex>

      <Heading mt={4} mb={1.5} size="sm">
        Language
      </Heading>
      <Select
        size="sm"
        bgColor={darkMode ? "#3c3c3c" : "white"}
        borderColor={darkMode ? "#3c3c3c" : "white"}
        value={language}
        onChange={(event) => onLanguageChange(event.target.value)}
      >
        {languages.map((lang) => (
          <option key={lang} value={lang} style={{ color: "black" }}>
            {lang}
          </option>
        ))}
      </Select>

      <Heading mt={4} mb={1.5} size="sm">
        Editor Profile
      </Heading>
      <Flex 
        p={3} 
        bg={darkMode ? "#3c3c3c" : "white"} 
        borderRadius="md" 
        fontSize="sm"
        alignItems="center"
      >
        <Box mr={3} color={darkMode ? "green.400" : "green.600"}>
          <Icon as={VscPerson} fontSize="lg" />
        </Box>
        <Text>
          <strong>{Object.keys(users).length + 1}</strong> active {Object.keys(users).length + 1 === 1 ? 'user' : 'users'}
        </Text>
      </Flex>

      <Heading mt={4} mb={1.5} size="sm">
        About
      </Heading>
      <Text fontSize="sm" mb={1.5}>
        <strong>Code Beautifier</strong> is a tool for formatting and improving 
        the indentation of your code for better readability.
      </Text>

      <Button
        size="sm"
        colorScheme={darkMode ? "whiteAlpha" : "blackAlpha"}
        borderColor={darkMode ? "purple.400" : "purple.600"}
        color={darkMode ? "purple.400" : "purple.600"}
        variant="outline"
        leftIcon={<VscRepo />}
        mt={1}
        onClick={onLoadSample}
      >
        View sample
      </Button>
    </Container>
  );
}

export default Sidebar;
