import { Box, Flex, HStack, Icon, Text, useToast } from "@chakra-ui/react";
import Editor from "@monaco-editor/react";
import { editor } from "monaco-editor/esm/vs/editor/editor.api";
import { useEffect, useRef, useState } from "react";
import { VscChevronRight, VscFolderOpened, VscGist, VscMenu } from "react-icons/vsc";
import useLocalStorageState from "use-local-storage-state";

import "./tooltip-fix.css";
import { javaSample } from "./javaSample";
import Footer from "./Footer";
import ReadCodeConfirm from "./ReadCodeConfirm";
import Sidebar from "./Sidebar";
import animals from "./animals.json";
import languages from "./languages.json";
import Rustpad, { UserInfo } from "./rustpad";
import useHash from "./useHash";

function getWsUri(id: string) {
  let url = new URL(`api/socket/${id}`, window.location.href);
  url.protocol = url.protocol == "https:" ? "wss:" : "ws:";
  return url.href;
}

function generateName() {
  return "Anonymous " + animals[Math.floor(Math.random() * animals.length)];
}

function generateHue() {
  return Math.floor(Math.random() * 360);
}

function App() {
  const toast = useToast();
  const [language, setLanguage] = useState("plaintext");
  const [connection, setConnection] = useState<
    "connected" | "disconnected" | "desynchronized"
  >("disconnected");
  const [users, setUsers] = useState<Record<number, UserInfo>>({});
  const [name, setName] = useLocalStorageState("name", {
    defaultValue: generateName,
  });
  const [hue, setHue] = useLocalStorageState("hue", {
    defaultValue: generateHue,
  });
  const [editor, setEditor] = useState<editor.IStandaloneCodeEditor>();
  const [darkMode, setDarkMode] = useLocalStorageState("darkMode", {
    defaultValue: false,
  });
  const [sidebarCollapsed, setSidebarCollapsed] = useLocalStorageState("sidebarCollapsed", {
    defaultValue: true,
  });
  const rustpad = useRef<Rustpad>();
  const id = useHash();

  const [readCodeConfirmOpen, setReadCodeConfirmOpen] = useState(false);

  useEffect(() => {
    if (editor?.getModel()) {
      const model = editor.getModel()!;
      model.setValue("");
      model.setEOL(0); // LF
      rustpad.current = new Rustpad({
        uri: getWsUri(id),
        editor,
        onConnected: () => setConnection("connected"),
        onDisconnected: () => setConnection("disconnected"),
        onDesynchronized: () => {
          setConnection("desynchronized");
          toast({
            title: "Formatting service unavailable",
            description: "Please save your work and refresh the page.",
            status: "error",
            duration: null,
          });
        },
        onChangeLanguage: (language) => {
          if (languages.includes(language)) {
            setLanguage(language);
          }
        },
        onChangeUsers: setUsers,
      });
      return () => {
        rustpad.current?.dispose();
        rustpad.current = undefined;
      };
    }
  }, [id, editor, toast, setUsers]);

  useEffect(() => {
    if (connection === "connected") {
      rustpad.current?.setInfo({ name, hue });
    }
  }, [connection, name, hue]);

  // Effect to handle persistent tooltips
  useEffect(() => {
    // Function to remove any visible tooltips on mousemove
    const handleMouseMove = (e: MouseEvent) => {
      const tooltips = document.querySelectorAll('.monaco-hover');
      if (tooltips.length > 0) {
        // After a short delay, remove tooltips that shouldn't be persistent
        setTimeout(() => {
          tooltips.forEach(tooltip => {
            // Check if the mouse is still over the tooltip
            const rect = tooltip.getBoundingClientRect();
            const mouseX = e.clientX;
            const mouseY = e.clientY;
            
            if (mouseX < rect.left || mouseX > rect.right || 
                mouseY < rect.top || mouseY > rect.bottom) {
              tooltip.remove();
            }
          });
        }, 300);
      }
    };

    // Add event listener for mouse movement
    document.addEventListener('mousemove', handleMouseMove);
    
    // Cleanup function to remove event listener
    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
    };
  }, []);

  function handleLanguageChange(language: string) {
    setLanguage(language);
    if (rustpad.current?.setLanguage(language)) {
      toast({
        title: "Language updated",
        description: (
          <>
            Switched to{" "}
            <Text as="span" fontWeight="semibold">
              {language}
            </Text>
            {" "}formatting.
          </>
        ),
        status: "info",
        duration: 2000,
        isClosable: true,
      });
    }
  }

  function handleLoadSample(confirmed: boolean) {
    if (editor?.getModel()) {
      const model = editor.getModel()!;
      const range = model.getFullModelRange();

      // If there are at least 10 lines of code, ask for confirmation.
      if (range.endLineNumber >= 10 && !confirmed) {
        setReadCodeConfirmOpen(true);
        return;
      }

      model.pushEditOperations(
        editor.getSelections(),
        [{ range, text: javaSample }],
        () => null,
      );
      editor.setPosition({ column: 0, lineNumber: 0 });
      if (language !== "java") {
        handleLanguageChange("java");
      }
    }
  }

  function handleDarkModeChange() {
    setDarkMode(!darkMode);
  }

  function handleSidebarToggle() {
    setSidebarCollapsed(!sidebarCollapsed);
  }

  return (
    <Flex
      direction="column"
      h="100vh"
      overflow="hidden"
      bgColor={darkMode ? "#1e1e1e" : "white"}
      color={darkMode ? "#cbcaca" : "inherit"}
    >
      <Box
        flexShrink={0}
        bgColor={darkMode ? "#333333" : "#e8e8e8"}
        color={darkMode ? "#cccccc" : "#383838"}
        textAlign="center"
        fontSize="sm"
        py={0.5}
      >
        Code Beautifier
      </Box>
      <Flex flex="1 0" minH={0}>
        <Sidebar
          connection={connection}
          darkMode={darkMode}
          language={language}
          currentUser={{ name, hue }}
          users={users}
          onDarkModeChange={handleDarkModeChange}
          onLanguageChange={handleLanguageChange}
          onLoadSample={() => handleLoadSample(false)}
          onChangeName={(name) => name.length > 0 && setName(name)}
          onChangeColor={() => setHue(generateHue())}
          collapsed={sidebarCollapsed}
          onToggle={handleSidebarToggle}
        />
        <ReadCodeConfirm
          isOpen={readCodeConfirmOpen}
          onClose={() => setReadCodeConfirmOpen(false)}
          onConfirm={() => {
            handleLoadSample(true);
            setReadCodeConfirmOpen(false);
          }}
        />

        <Flex flex={1} minW={0} h="100%" direction="column" overflow="hidden">
          <HStack
            h={6}
            spacing={1}
            color="#888888"
            fontWeight="medium"
            fontSize="13px"
            px={3.5}
            flexShrink={0}
            position="relative"
          >
            <Icon as={VscFolderOpened} fontSize="md" color="blue.500" />
            <Text>documents</Text>
            <Icon as={VscChevronRight} fontSize="md" />
            <Icon as={VscGist} fontSize="md" color="purple.500" />
            <Text>{id}</Text>
            
            {/* Connection status indicator at top to replace any automatic tooltips */}
            {connection === "connected" && (
              <Box 
                position="absolute" 
                top="0" 
                left="0" 
                right="0" 
                bgColor="rgba(0,0,0,0.7)" 
                color="white"
                fontSize="xs"
                p={1}
                display="none"
              >
                You are connected!
              </Box>
            )}
          </HStack>
          <Box flex={1} minH={0}>
            <Editor
              theme={darkMode ? "vs-dark" : "vs"}
              language={language}
              options={{
                automaticLayout: true,
                fontSize: 13,
                hover: {
                  enabled: false,
                  delay: 100,
                  sticky: false
                }
              }}
              beforeMount={(monaco) => {
                // Completely disable hover widgets
                monaco.editor.registerCommand('editor.action.showHover', () => {
                  // Do nothing, effectively disabling hover
                  return null;
                });
              }}
              onMount={(editor) => setEditor(editor)}
            />
          </Box>
        </Flex>
      </Flex>
      <Footer users={users} />
    </Flex>
  );
}

export default App;
