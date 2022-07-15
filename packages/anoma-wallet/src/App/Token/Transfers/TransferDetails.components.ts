import styled from "styled-components/macro";

export const TransferDetailContainer = styled.div`
  display: flex;
  flex-direction: column;
  justify-content: start;
  align-items: start;
  width: 100%;
  height: 100%;
  color: ${(props) => props.theme.colors.titleColor};
`;

export const Address = styled.pre`
  color: ${(props) => props.theme.colors.inputText};
  background-color: ${(props) => props.theme.colors.inputBackground};
  font-size: 12px;
  text-align: center;
  margin: 0 0 20px 0;
  padding: 4px 8px;
  border-radius: 4px;
  scrollbar-width: 0;
  white-space: pre-wrap;
  word-wrap: break-word;

  ::-webkit-scrollbar {
    display: none;
  }
`;
