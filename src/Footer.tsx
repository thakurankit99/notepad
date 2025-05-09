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
      h={{ base: "36px", md: "22px" }} 
      bgColor="#0071c3" 
      color="white"
      justifyContent="space-between"
      alignItems="center"
      width="100%"
      position="relative"
      zIndex={10}
      boxShadow="0 -1px 2px rgba(0, 0, 0, 0.2)"
      minH={{ base: "36px", md: "22px" }}
      css={{
        '@supports (padding-bottom: env(safe-area-inset-bottom))': {
          paddingBottom: 'env(safe-area-inset-bottom)'
        }
      }}
    >
      <Flex
        h="100%"
        bgColor="#09835c"
        pl={{ base: 3, md: 2.5 }}
        pr={{ base: 5, md: 4 }}
        fontSize={{ base: "sm", md: "xs" }}
        align="center"
        minH="100%"
      >
        <Icon as={VscRemote} mb={-0.5} mr={{ base: 2, md: 1 }} fontSize="16px" />
        <Text fontSize={{ base: "sm", md: "xs" }} fontWeight="medium">Code Beautifier ({activeUsers})</Text>
      </Flex>
      
      {/* Font size controls */}
      <Flex 
        h="100%"
        minH="100%" 
        pr={{ base: 4, md: 3 }}
        pl={{ base: 2, md: 1 }}
        alignItems="center"
        justify="flex-end"
      >
        <IconButton
          aria-label="Decrease font size"
          icon={<VscRemove fontSize="14px" />}
          size="sm"
          height={{ base: "26px", md: "18px" }}
          minWidth={{ base: "26px", md: "18px" }}
          variant="ghost"
          onClick={handleDecrease}
          colorScheme="whiteAlpha"
          color="white"
          opacity={0.9}
          _hover={{ opacity: 1 }}
          mr={{ base: 2, md: 1 }}
          sx={{ touchAction: "manipulation" }}
        />
        
        <Flex width={{ base: "100px", md: "80px" }} alignItems="center" px={2}>
          <Slider 
            min={MIN_FONT_SIZE} 
            max={MAX_FONT_SIZE} 
            step={1} 
            value={fontSize} 
            onChange={handleSliderChange}
            size="md"
          >
            <SliderTrack bg="whiteAlpha.300" h={{ base: "4px", md: "2px" }}>
              <SliderFilledTrack bg="whiteAlpha.700" />
            </SliderTrack>
            <SliderThumb boxSize={{ base: 4, md: 2 }} bg="white" />
          </Slider>
        </Flex>
        
        <IconButton
          aria-label="Increase font size"
          icon={<VscAdd fontSize="14px" />}
          size="sm"
          height={{ base: "26px", md: "18px" }}
          minWidth={{ base: "26px", md: "18px" }}
          variant="ghost"
          onClick={handleIncrease}
          colorScheme="whiteAlpha"
          color="white"
          opacity={0.9}
          _hover={{ opacity: 1 }}
          ml={{ base: 2, md: 1 }}
          sx={{ touchAction: "manipulation" }}
        />
        
        <Text fontSize={{ base: "xs", md: "10px" }} ml={2} opacity={0.9} fontWeight="medium">
          {fontSize}px
        </Text>
      </Flex>
    </Flex>
  );
}

export default Footer;
