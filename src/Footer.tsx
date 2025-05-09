import { 
  Flex, 
  Icon, 
  Text, 
  IconButton, 
  Slider, 
  SliderTrack, 
  SliderFilledTrack, 
  SliderThumb, 
  Box,
  Tooltip
} from "@chakra-ui/react";
import { VscRemote, VscAdd, VscRemove, VscSearch } from "react-icons/vsc";
import { UserInfo } from "./rustpad";
import { editor } from "monaco-editor/esm/vs/editor/editor.api";

type FooterProps = {
  users: Record<number, UserInfo>;
  fontSize: number;
  setFontSize: (size: number) => void;
  editor?: editor.IStandaloneCodeEditor;
  darkMode: boolean;
  setShowSearch?: React.Dispatch<React.SetStateAction<boolean>>;
};

function Footer({ users, fontSize, setFontSize, editor, darkMode, setShowSearch }: FooterProps) {
  const activeUsers = Object.keys(users).length + 1;
  
  const MIN_FONT_SIZE = 8;
  const MAX_FONT_SIZE = 24;
  
  const handleIncrease = () => {
    const newSize = Math.min(fontSize + 1, MAX_FONT_SIZE);
    setFontSize(newSize);
    if (editor) {
      editor.updateOptions({ fontSize: newSize });
    }
  };

  const handleDecrease = () => {
    const newSize = Math.max(fontSize - 1, MIN_FONT_SIZE);
    setFontSize(newSize);
    if (editor) {
      editor.updateOptions({ fontSize: newSize });
    }
  };
  
  const handleSliderChange = (value: number) => {
    setFontSize(value);
    if (editor) {
      editor.updateOptions({ fontSize: value });
    }
  };
  
  // Handle search button click
  const handleOpenSearch = () => {
    if (setShowSearch) {
      setShowSearch(true);
    }
  };
  
  return (
    <Flex 
      h={{ base: "28px", md: "22px" }} 
      bgColor="#0071c3" 
      color="white"
      justifyContent="space-between"
      alignItems="center"
      width="100%"
      position="relative"
      zIndex={10}
      boxShadow="0 -1px 2px rgba(0, 0, 0, 0.2)"
      minH={{ base: "28px", md: "22px" }}
      css={{
        '@supports (padding-bottom: env(safe-area-inset-bottom))': {
          paddingBottom: 'env(safe-area-inset-bottom)'
        }
      }}
    >
      <Flex
        h="100%"
        bgColor="#09835c"
        pl={{ base: 2, md: 2.5 }}
        pr={{ base: 3, md: 4 }}
        fontSize={{ base: "xs", md: "xs" }}
        align="center"
        minH="100%"
      >
        <Icon as={VscRemote} mb={-0.5} mr={{ base: 1, md: 1 }} fontSize="14px" />
        <Text fontSize="xs" fontWeight="medium">Code Beautifier ({activeUsers})</Text>
      </Flex>
      
      <Flex
        h="100%"
        minH="100%" 
        pr={{ base: 2, md: 3 }}
        pl={{ base: 1, md: 1 }}
        alignItems="center"
        justify="flex-end"
      >
        {/* Add Search Button */}
        <Tooltip label="Search (Ctrl+F)" placement="top">
          <IconButton
            aria-label="Search"
            icon={<VscSearch fontSize="10px" />}
            size="xs"
            height={{ base: "20px", md: "18px" }}
            minWidth={{ base: "20px", md: "18px" }}
            variant="ghost"
            onClick={handleOpenSearch}
            colorScheme="whiteAlpha"
            color="white"
            opacity={0.9}
            _hover={{ opacity: 1 }}
            mr={{ base: 2, md: 2 }}
            sx={{ touchAction: "manipulation" }}
          />
        </Tooltip>
        
        <IconButton
          aria-label="Decrease font size"
          icon={<VscRemove fontSize="10px" />}
          size="xs"
          height={{ base: "20px", md: "18px" }}
          minWidth={{ base: "20px", md: "18px" }}
          variant="ghost"
          onClick={handleDecrease}
          colorScheme="whiteAlpha"
          color="white"
          opacity={0.9}
          _hover={{ opacity: 1 }}
          mr={{ base: 1, md: 1 }}
          sx={{ touchAction: "manipulation" }}
        />
        
        <Flex width={{ base: "70px", md: "80px" }} alignItems="center" px={{ base: 1, md: 2 }}>
          <Slider 
            min={MIN_FONT_SIZE} 
            max={MAX_FONT_SIZE} 
            step={1} 
            value={fontSize} 
            onChange={handleSliderChange}
            size={{ base: "sm", md: "md" }}
          >
            <SliderTrack bg="whiteAlpha.300" h={{ base: "2px", md: "2px" }}>
              <SliderFilledTrack bg="whiteAlpha.700" />
            </SliderTrack>
            <SliderThumb boxSize={{ base: 3, md: 2 }} bg="white" />
          </Slider>
        </Flex>
        
        <IconButton
          aria-label="Increase font size"
          icon={<VscAdd fontSize="10px" />}
          size="xs"
          height={{ base: "20px", md: "18px" }}
          minWidth={{ base: "20px", md: "18px" }}
          variant="ghost"
          onClick={handleIncrease}
          colorScheme="whiteAlpha"
          color="white"
          opacity={0.9}
          _hover={{ opacity: 1 }}
          ml={{ base: 1, md: 1 }}
          sx={{ touchAction: "manipulation" }}
        />
        
        <Text fontSize="10px" ml={{ base: 1, md: 2 }} opacity={0.9} fontWeight="medium">
          {fontSize}px
        </Text>
      </Flex>
    </Flex>
  );
}

export default Footer;
