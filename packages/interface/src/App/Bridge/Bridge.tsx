import { NavigationContainer } from "components/NavigationContainer";
import { Heading, HeadingLevel } from "components/Heading";
import { BridgeContainer } from "./Bridge.components";
import IBCTransfer from "App/Token/IBCTransfer/IBCTransfer";

export const Bridge = (): JSX.Element => {
  return (
    <BridgeContainer>
      <NavigationContainer>
        <Heading level={HeadingLevel.One}>Bridge</Heading>
      </NavigationContainer>
      <IBCTransfer />
    </BridgeContainer>
  );
};