import { Flex, Float, Spinner, Text } from "@chakra-ui/react";

const LoadingPopup: React.FC<{ loading: boolean }> = ({ loading }) => {
  if (!loading) {
    return <div />;
  }
  return (
    <Float placement="middle-center">
      <Flex bg="white" p="4" direction="column" alignItems="center" gap="2">
        <Spinner color="black" size="xl" />
        <Text color="black">Loading...</Text>
      </Flex>
    </Float>
  );
};

export default LoadingPopup;
