import { 
  Flex, 
  Icon, 
  Text, 
  IconButton, 
  Slider, 
  SliderTrack, 
  SliderFilledTrack, 
  SliderThumb, 
  Box 
} from "@chakra-ui/react";
import { VscRemote, VscAdd, VscRemove } from "react-icons/vsc";
import { UserInfo } from "./rustpad";
import { editor } from "monaco-editor/esm/vs/editor/editor.api";

type FooterProps = {
  users: Record<number, UserInfo>;
  fontSize: number;
  setFontSize: (size: number) => void;
  editor?: editor.IStandaloneCodeEditor;
  darkMode: boolean;
};

function Footer({ users, fontSize, setFontSize, editor, darkMode }: FooterProps) {
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
  
  return (
    <Flex 
      h="22px" 
      bgColor="#0071c3" 
      color="white"
      justifyContent="space-between"
      alignItems="center"
    >
      <Flex
        h="100%"
        bgColor="#09835c"
        pl={2.5}
        pr={4}
        fontSize="sm"
        align="center"
      >
        <Icon as={VscRemote} mb={-0.5} mr={1} />
        <Text fontSize="xs">Code Beautifier ({activeUsers})</Text>
      </Flex>
      
      {/* Font size controls */}
      <Flex 
        h="100%" 
        pr={3} 
        alignItems="center"
        justify="flex-end"
      >
        <IconButton
          aria-label="Decrease font size"
          icon={<VscRemove fontSize="10px" />}
          size="xs"
          height="18px"
          minWidth="18px"
          variant="ghost"
          onClick={handleDecrease}
          colorScheme="whiteAlpha"
          color="white"
          opacity={0.8}
          _hover={{ opacity: 1 }}
          mr={1}
        />
        
        <Flex width="80px" alignItems="center" px={2}>
          <Slider 
            min={MIN_FONT_SIZE} 
            max={MAX_FONT_SIZE} 
            step={1} 
            value={fontSize} 
            onChange={handleSliderChange}
            size="sm"
          >
            <SliderTrack bg="whiteAlpha.300">
              <SliderFilledTrack bg="whiteAlpha.700" />
            </SliderTrack>
            <SliderThumb boxSize={2} bg="white" />
          </Slider>
        </Flex>
        
        <IconButton
          aria-label="Increase font size"
          icon={<VscAdd fontSize="10px" />}
          size="xs"
          height="18px"
          minWidth="18px"
          variant="ghost"
          onClick={handleIncrease}
          colorScheme="whiteAlpha"
          color="white"
          opacity={0.8}
          _hover={{ opacity: 1 }}
          ml={1}
        />
        
        <Text fontSize="10px" ml={2} opacity={0.9}>
          {fontSize}px
        </Text>
      </Flex>
    </Flex>
  );
}

export default Footer;
