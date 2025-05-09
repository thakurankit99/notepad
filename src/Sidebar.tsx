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
  IconButton,
  Tooltip,
} from "@chakra-ui/react";
import { VscRepo, VscMenu, VscChevronLeft, VscColorMode } from "react-icons/vsc";

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
  collapsed: boolean;
  onToggle: () => void;
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
  collapsed,
  onToggle,
}: SidebarProps) {
  const toast = useToast();

  if (collapsed) {
    return (
      <Box 
        h="100%" 
        bgColor={darkMode ? "#252526" : "#f3f3f3"} 
        display="flex"
        flexDirection="column"
        alignItems="center"
        py={4}
        px={2}
        width="50px"
        transition="width 0.3s ease"
        borderRight={darkMode ? "1px solid #444" : "1px solid #ddd"}
        boxShadow={darkMode ? "0 0 8px rgba(0,0,0,0.3)" : "0 0 5px rgba(0,0,0,0.1)"}
        zIndex={5}
      >
        <IconButton
          aria-label="Expand sidebar"
          icon={<VscMenu fontSize="18px" />}
          variant="ghost"
          size="sm"
          onClick={onToggle}
          mb={4}
          color={darkMode ? "white" : "gray.700"}
          bg={darkMode ? "whiteAlpha.100" : "transparent"}
          _hover={{ bg: darkMode ? "whiteAlpha.300" : "blackAlpha.100" }}
        />
        <ConnectionStatus darkMode={darkMode} connection={connection} isCollapsed />
        
        <Tooltip 
          label={darkMode ? "Switch to light mode" : "Switch to dark mode"} 
          placement="right"
          hasArrow
          closeOnClick={false}
          openDelay={300}
          closeOnEsc={true}
          gutter={10}
        >
          <IconButton
            aria-label="Toggle dark mode"
            icon={<VscColorMode fontSize="18px" />}
            variant="ghost"
            size="sm"
            onClick={onDarkModeChange}
            mt={4}
            color={darkMode ? "yellow.400" : "blue.600"}
            _hover={{ bg: darkMode ? "whiteAlpha.200" : "blackAlpha.100" }}
          />
        </Tooltip>
        
        <Tooltip 
          label={`Current language: ${language}`} 
          placement="right"
          hasArrow
          closeOnClick={false}
          openDelay={300}
          closeOnEsc={true}
          gutter={10}
        >
          <Flex 
            mt={4}
            p={2}
            borderRadius="md"
            fontSize="sm"
            alignItems="center"
            justifyContent="center"
            flexDirection="column"
          >
            <Text 
              fontSize="xs" 
              fontWeight="bold" 
              bgColor={darkMode ? "gray.700" : "gray.200"}
              color={darkMode ? "gray.200" : "gray.700"}
              px={2}
              py={1}
              borderRadius="md"
            >
              {language.slice(0, 2).toUpperCase()}
            </Text>
          </Flex>
        </Tooltip>
        
        <Tooltip 
          label="View Java sample code" 
          placement="right"
          hasArrow
          closeOnClick={false}
          openDelay={300}
          closeOnEsc={true}
          gutter={10}
        >
          <IconButton
            aria-label="View sample"
            icon={<VscRepo fontSize="18px" />}
            variant="ghost"
            size="sm"
            onClick={onLoadSample}
            mt={4}
            color={darkMode ? "purple.400" : "purple.600"}
            _hover={{ bg: darkMode ? "whiteAlpha.200" : "blackAlpha.100" }}
          />
        </Tooltip>
      </Box>
    );
  }

  return (
    <Container
      w={{ base: "3xs", md: "2xs", lg: "xs" }}
      display={{ base: "block", sm: "block" }}
      bgColor={darkMode ? "#252526" : "#f3f3f3"}
      overflowY="auto"
      maxW="full"
      lineHeight={1.4}
      py={4}
      position="relative"
      transition="width 0.3s ease"
      borderRight={darkMode ? "1px solid #444" : "1px solid #ddd"}
      zIndex={5}
    >
      <Flex justifyContent="space-between" alignItems="center" mb={4}>
        <ConnectionStatus darkMode={darkMode} connection={connection} />
        <IconButton
          aria-label="Collapse sidebar"
          icon={<VscChevronLeft fontSize="18px" />}
          variant="ghost"
          size="sm"
          onClick={onToggle}
          color={darkMode ? "white" : "gray.700"}
          _hover={{ bg: darkMode ? "whiteAlpha.200" : "blackAlpha.100" }}
        />
      </Flex>

      <Flex justifyContent="space-between" mt={4} mb={1.5} w="full">
        <Heading size="sm">Dark Mode</Heading>
        <Switch isChecked={darkMode} onChange={onDarkModeChange} />
      </Flex>

      <Text fontSize="xs" color={darkMode ? "gray.400" : "gray.500"} mb={3}>
        Using {darkMode ? "dark" : "light"} mode - matches your system preference unless manually set
      </Text>

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
        About
      </Heading>
      <Text fontSize="sm" mb={1.5}>
        <strong>Code Beautifier</strong> is a tool for formatting and improving 
        the indentation of your code for better readability.
      </Text>
      
      <Text fontSize="xs" color={darkMode ? "gray.400" : "gray.500"} mt={1} mb={2}>
        Use the +/- slider in the footer to adjust text size
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
