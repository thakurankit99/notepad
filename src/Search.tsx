import {
  Box,
  Input,
  InputGroup,
  InputRightElement,
  IconButton,
  Flex,
  Text,
  Tooltip,
  HStack,
  Collapse
} from "@chakra-ui/react";
import { useRef, useState, useEffect } from "react";
import { editor, KeyMod, KeyCode } from "monaco-editor/esm/vs/editor/editor.api";
import { 
  VscSearch, 
  VscClose, 
  VscArrowDown, 
  VscArrowUp,
  VscRegex, 
  VscCaseSensitive, 
  VscWholeWord, 
  VscReplaceAll
} from "react-icons/vsc";

interface SearchProps {
  editor?: editor.IStandaloneCodeEditor;
  darkMode: boolean;
  setShowSearch?: React.Dispatch<React.SetStateAction<boolean>>;
}

const Search: React.FC<SearchProps> = ({ editor, darkMode, setShowSearch }) => {
  const [isOpen, setIsOpen] = useState<boolean>(false);
  const [searchText, setSearchText] = useState<string>("");
  const [replaceText, setReplaceText] = useState<string>("");
  const [showReplace, setShowReplace] = useState<boolean>(false);
  const [matchCase, setMatchCase] = useState<boolean>(false);
  const [useRegex, setUseRegex] = useState<boolean>(false);
  const [matchWholeWord, setMatchWholeWord] = useState<boolean>(false);
  
  const searchInputRef = useRef<HTMLInputElement>(null);
  
  // Focus search input when opened
  useEffect(() => {
    if (isOpen && searchInputRef.current) {
      setTimeout(() => {
        searchInputRef.current?.focus();
      }, 100);
    }
  }, [isOpen]);
  
  // Toggle search panel visibility with keyboard shortcut
  useEffect(() => {
    if (!editor) return;
    
    // Add keyboard shortcut (Ctrl/Cmd + F)
    const disposable = editor.addCommand(
      KeyMod.CtrlCmd | KeyCode.KeyF,
      () => {
        setIsOpen(true);
      }
    );
    
    return () => {
      // Check if disposable exists and has a dispose method
      if (disposable && typeof disposable === 'object' && 'dispose' in disposable) {
        disposable.dispose();
      }
    };
  }, [editor]);
  
  // Find in current editor
  const findInEditor = () => {
    if (!editor || !searchText) return;
    
    // Find all matches
    const model = editor.getModel();
    if (!model) return;
    
    // Execute find action
    const findAction = editor.getAction("actions.find");
    if (findAction) {
      findAction.run();
    }
    
    // Set find options in editor
    const findController = editor.getContribution("editor.contrib.findController") as any;
    if (findController) {
      findController.setOptions({
        searchString: searchText,
        isRegex: useRegex,
        matchCase: matchCase,
        wholeWord: matchWholeWord
      });
    }
  };
  
  // Handle search input
  const handleSearchChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setSearchText(e.target.value);
  };
  
  // Find next match
  const findNext = () => {
    if (editor && searchText) {
      const findController = editor.getContribution("editor.contrib.findController") as any;
      if (findController) {
        findController.moveToNextMatch();
      }
    }
  };
  
  // Find previous match
  const findPrevious = () => {
    if (editor && searchText) {
      const findController = editor.getContribution("editor.contrib.findController") as any;
      if (findController) {
        findController.moveToPrevMatch();
      }
    }
  };
  
  // Replace current match
  const replaceCurrent = () => {
    if (editor && searchText) {
      const findController = editor.getContribution("editor.contrib.findController") as any;
      if (findController) {
        findController.replace();
      }
    }
  };
  
  // Replace all matches
  const replaceAll = () => {
    if (editor && searchText) {
      const findController = editor.getContribution("editor.contrib.findController") as any;
      if (findController) {
        findController.replaceAll();
      }
    }
  };
  
  // Close search panel and reset state
  const closeSearch = () => {
    setIsOpen(false);
    if (setShowSearch) {
      setShowSearch(false);
    }
    const findController = editor?.getContribution("editor.contrib.findController") as any;
    if (findController) {
      findController.closeFindWidget();
    }
  };
  
  // Toggle replace mode
  const toggleReplace = () => {
    setShowReplace(!showReplace);
  };

  // Immediate search on criteria change
  useEffect(() => {
    if (searchText && editor) {
      findInEditor();
    }
  }, [searchText, matchCase, useRegex, matchWholeWord]);
  
  if (!isOpen) return null;
  
  return (
    <Box
      position="absolute"
      top={2}
      right="50%"
      transform="translateX(50%)"
      zIndex={100}
      width="320px"
      bg={darkMode ? "gray.800" : "white"}
      boxShadow="md"
      borderRadius="md"
      overflow="hidden"
    >
      <Box p={2}>
        <Flex justify="space-between" mb={2}>
          <HStack spacing={1}>
            <Tooltip label="Match Case">
              <IconButton
                aria-label="Match Case"
                icon={<VscCaseSensitive />}
                size="sm"
                variant={matchCase ? "solid" : "ghost"}
                colorScheme={matchCase ? "blue" : darkMode ? "whiteAlpha" : "gray"}
                onClick={() => setMatchCase(!matchCase)}
              />
            </Tooltip>
            
            <Tooltip label="Use Regular Expression">
              <IconButton
                aria-label="Use Regular Expression"
                icon={<VscRegex />}
                size="sm"
                variant={useRegex ? "solid" : "ghost"}
                colorScheme={useRegex ? "blue" : darkMode ? "whiteAlpha" : "gray"}
                onClick={() => setUseRegex(!useRegex)}
              />
            </Tooltip>
            
            <Tooltip label="Match Whole Word">
              <IconButton
                aria-label="Match Whole Word"
                icon={<VscWholeWord />}
                size="sm"
                variant={matchWholeWord ? "solid" : "ghost"}
                colorScheme={matchWholeWord ? "blue" : darkMode ? "whiteAlpha" : "gray"}
                onClick={() => setMatchWholeWord(!matchWholeWord)}
              />
            </Tooltip>
          </HStack>
          
          <HStack>
            <Text fontSize="xs" cursor="pointer" color={showReplace ? "blue.500" : darkMode ? "gray.300" : "gray.600"} onClick={toggleReplace}>
              Replace
            </Text>
            <IconButton
              aria-label="Close Search"
              icon={<VscClose />}
              size="sm"
              variant="ghost"
              onClick={closeSearch}
            />
          </HStack>
        </Flex>
        
        <InputGroup size="sm" mb={showReplace ? 2 : 0}>
          <Input
            ref={searchInputRef}
            placeholder="Search"
            value={searchText}
            onChange={handleSearchChange}
            bg={darkMode ? "gray.700" : "white"}
            borderColor={darkMode ? "gray.600" : "gray.200"}
            _focus={{
              borderColor: "blue.500",
              boxShadow: "0 0 0 1px blue.500",
            }}
            onKeyPress={(e) => {
              if (e.key === "Enter") {
                e.shiftKey ? findPrevious() : findNext();
              }
            }}
          />
          <InputRightElement width="60px">
            <IconButton
              aria-label="Previous Match"
              icon={<VscArrowUp />}
              size="xs"
              variant="ghost"
              onClick={findPrevious}
              mr={-1}
            />
            <IconButton
              aria-label="Next Match"
              icon={<VscArrowDown />}
              size="xs"
              variant="ghost"
              onClick={findNext}
            />
          </InputRightElement>
        </InputGroup>
        
        <Collapse in={showReplace} animateOpacity>
          <Box pt={1}>
            <InputGroup size="sm">
              <Input
                placeholder="Replace"
                value={replaceText}
                onChange={(e) => setReplaceText(e.target.value)}
                bg={darkMode ? "gray.700" : "white"}
                borderColor={darkMode ? "gray.600" : "gray.200"}
                _focus={{
                  borderColor: "blue.500",
                  boxShadow: "0 0 0 1px blue.500",
                }}
                onKeyPress={(e) => {
                  if (e.key === "Enter") {
                    replaceCurrent();
                  }
                }}
              />
              <InputRightElement width="30px">
                <Tooltip label="Replace All">
                  <IconButton
                    aria-label="Replace All"
                    icon={<VscReplaceAll />}
                    size="xs"
                    variant="ghost"
                    onClick={replaceAll}
                  />
                </Tooltip>
              </InputRightElement>
            </InputGroup>
          </Box>
        </Collapse>
      </Box>
    </Box>
  );
};

export default Search; 