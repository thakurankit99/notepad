import { Box, Flex, HStack, Icon, Text, useToast, IconButton, Tooltip } from "@chakra-ui/react";
import Editor from "@monaco-editor/react";
import { editor } from "monaco-editor/esm/vs/editor/editor.api";
import { useEffect, useRef, useState } from "react";
import { VscGist, VscMenu, VscAdd, VscRemove } from "react-icons/vsc";
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

// Add function to detect system theme preference
function detectSystemTheme(): boolean {
  if (typeof window === 'undefined') return false;
  return window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches;
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
  
  // Use system theme as default
  const [darkMode, setDarkMode] = useLocalStorageState("darkMode", {
    defaultValue: detectSystemTheme(),
  });
  
  // Add font size state with localStorage persistence
  const [fontSize, setFontSize] = useLocalStorageState("fontSize", {
    defaultValue: 13
  });
  
  const [sidebarCollapsed, setSidebarCollapsed] = useLocalStorageState("sidebarCollapsed", {
    defaultValue: true,
  });
  const rustpad = useRef<Rustpad>();
  const id = useHash();

  const [readCodeConfirmOpen, setReadCodeConfirmOpen] = useState(false);

  // Listen for system theme changes
  useEffect(() => {
    if (!window.matchMedia) return;
    
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    
    const handleChange = (e: MediaQueryListEvent) => {
      // Only update if the user hasn't explicitly set a preference yet
      // Check if localStorage has been written to by user interaction
      if (!localStorage.getItem('userThemePreference')) {
        setDarkMode(e.matches);
      }
    };
    
    // Modern browsers
    mediaQuery.addEventListener('change', handleChange);
    
    return () => {
      mediaQuery.removeEventListener('change', handleChange);
    };
  }, [setDarkMode]);

  // Add keyboard shortcuts for text zoom (Ctrl/Cmd + +/-)
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Check if Ctrl key or Meta key (Command on Mac) is pressed
      if (e.ctrlKey || e.metaKey) {
        if (e.key === '+' || e.key === '=') {
          e.preventDefault(); // Prevent browser zoom
          increaseTextSize();
        } else if (e.key === '-' || e.key === '_') {
          e.preventDefault(); // Prevent browser zoom
          decreaseTextSize();
        }
      }
    };
    
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [fontSize]); // Re-add listener when fontSize changes to ensure correct state

  // Function to copy current URL to clipboard
  const copyUrlToClipboard = () => {
    try {
      navigator.clipboard.writeText(window.location.href);
      // No notification shown as per requirements
    } catch (err) {
      console.error("Failed to copy URL to clipboard", err);
    }
  };

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

  // Text zoom functions
  const increaseTextSize = () => {
    const newSize = Math.min(fontSize + 1, 24); // Maximum font size
    setFontSize(newSize);
    if (editor) {
      editor.updateOptions({ fontSize: newSize });
    }
  };

  const decreaseTextSize = () => {
    const newSize = Math.max(fontSize - 1, 8); // Minimum font size
    setFontSize(newSize);
    if (editor) {
      editor.updateOptions({ fontSize: newSize });
    }
  };

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
    // When user explicitly changes theme, record that preference
    localStorage.setItem('userThemePreference', 'set');
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
      <Flex
        flexShrink={0}
        bgColor={darkMode ? "#333333" : "#e8e8e8"}
        color={darkMode ? "#cccccc" : "#383838"}
        fontSize="sm"
        py={0.5}
        px={3}
        alignItems="center"
        justifyContent="center"
        position="relative"
      >
        <Flex 
          position="absolute" 
          left={3} 
          alignItems="center" 
          _hover={{ cursor: "pointer", opacity: 0.8 }}
          onClick={copyUrlToClipboard}
        >
          <Icon as={VscGist} fontSize="md" color="purple.500" />
          <Text ml={1} fontSize="xs" fontWeight="medium">{id}</Text>
        </Flex>
        <Text fontWeight="medium">Code Beautifier</Text>
        
        {/* Add font size controls */}
        <Flex 
          position="absolute" 
          right={3} 
          alignItems="center"
        >
          <Tooltip label="Decrease font size (Ctrl/Cmd + -)" openDelay={500}>
            <IconButton
              aria-label="Decrease font size"
              icon={<VscRemove />}
              size="xs"
              variant="ghost"
              onClick={decreaseTextSize}
              mr={1}
              color={darkMode ? "gray.300" : "gray.600"}
            />
          </Tooltip>
          <Tooltip label="Current font size">
            <Text fontSize="xs" mx={1} minW="20px" textAlign="center">
              {fontSize}
            </Text>
          </Tooltip>
          <Tooltip label="Increase font size (Ctrl/Cmd + +)" openDelay={500}>
            <IconButton
              aria-label="Increase font size"
              icon={<VscAdd />}
              size="xs"
              variant="ghost"
              onClick={increaseTextSize}
              ml={1}
              color={darkMode ? "gray.300" : "gray.600"}
            />
          </Tooltip>
        </Flex>
      </Flex>
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
          <Box flex={1} minH={0}>
            <Editor
              theme={darkMode ? "vs-dark" : "vs"}
              language={language}
              options={{
                automaticLayout: true,
                fontSize: fontSize,
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
