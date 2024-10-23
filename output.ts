export type ActionConfigMemberType = 'ai' | 'manual' | '%future added value';

export type ActionConfigurationMember = {
  __typename?: 'ActionConfigurationMember';
  /**
   * Whether additional array entries are allowed
   */
  allowAdditionalArrayEntries: boolean;
  /**
   * The manual value for the member
   */
  manual: ActionConfigurationMemberManual | null;
  /**
   * The path to the value
   */
  path: Array<string>;
  /**
   * The prompt for the member
   */
  prompt: ActionConfigurationMemberPrompt | null;
  /**
   * The type of the member
   */
  type: ActionConfigMemberType;
  /**
   * The union type of the member
   */
  unionType: number | null;
};

export type ActionConfigurationMemberManual = unknown;

export type ActionConfigurationMemberPrompt = unknown;

export type ActionDetails = Node & {
  __typename?: 'ActionDetails';
  /**
   * The auth config of the action, null if no auth is required.
   */
  authConfig: AuthConfig | null;
  dependencies: Array<string>;
  /**
   * The description of a tool, to be shown in the frontend
   */
  description: string;
  /**
   * The display name of a tool, to be shown in the frontend.
   */
  displayName: string;
  dynamicLoaders: DynamicLoaders | null;
  /**
   * Whether the action has side effects
   */
  hasSideEffects: boolean;
  inputSchema: JSON | null;
  /**
   * Whether the action is premium
   */
  isPremium: boolean;
  /**
   * The key of the action.
   */
  key: string;
  nodeId: string;
  /**
   * The observable channel of the action
   */
  observableChannel: ObservableChannelMetadata | null;
  /**
   * The output properties of the action
   */
  outputProperties: Array<ActionProperty>;
  outputSchema: JSON | null;
  tool: LindyTool;
  /**
   * The display name of a tool, to be shown in the frontend.
   */
  toolDisplayName: string;
};

export type ActionNode = Node & StateGraphNode & {
  __typename?: 'ActionNode';
  /**
   * The action configuration of the action
   */
  actionConfiguration: Array<ActionConfigurationMember>;
  /**
   * The action of the node
   */
  actionDefinition: StateGraphAction | null;
  /**
   * The auth of the action
   */
  auth: Auth | null;
  /**
   * The auth entries available for the action
   */
  authList: Array<Auth>;
  /**
   * The user-defined display name of the node
   */
  displayName: string | null;
  /**
   * The guidelines of the action
   */
  guidelines: string | null;
  /**
   * The unique identifier for the node
   */
  id: string;
  /**
   * The knowledge base sources of the action
   */
  knowledgeSources: Array<KnowledgeSource>;
  /**
   * Whether the node is manually positioned
   */
  manuallyPositioned: boolean;
  /**
   * The LLM model of the action
   */
  model: string;
  /**
   * The globally unique ID for the Action Node
   */
  nodeId: string;
  /**
   * The occurrence of the action
   */
  occurrence: number;
  /**
   * The position of the node
   */
  position: NodePosition;
  /**
   * Whether the action requires confirmation
   */
  requireConfirmation: AgentRequireConfirmation;
  /**
   * The stateful tool occurrence of the action
   */
  statefulToolOccurrence: number | null;
  /**
   * The type of the node
   */
  type: NodeType;
};

export type ActionProperty = {
  __typename?: 'ActionProperty';
  /**
   * The optional display name for the action property.
   */
  displayName: string | null;
  /**
   * The type of the action property.
   */
  type: ActionPropertyType;
  /**
   * The value of the action property.
   */
  value: string;
};

export type ActionPropertyType = 'Array' | 'Boolean' | 'Integer' | 'Null' | 'Number' | 'Object' | 'String' | 'Unknown' | '%future added value';

/**
 * An Agent Definition is the same as a Lindy, it contains information about the agent's behavior
 */
export type AgentDefinition = Node & {
  __typename?: 'AgentDefinition';
  conversations: AgentDefinitionConversationsConnection;
  /**
   * The date the agent was created
   */
  createdAt: DateTime;
  /**
   * Whether the agent is enabled or not. If not enabled tasks will not be executed for this agent
   */
  enabled: boolean;
  /**
   * Greeting message for a new conversation
   */
  greetingMessage: string;
  hasPremiumActions: boolean;
  hasPremiumTriggers: boolean;
  icon: AgentIcon | null;
  id: string;
  /**
   * General Guidelines of the Agent Definition
   */
  instructions: string;
  /**
   * Integrations that the agent is connected to
   */
  integrations: Array<Tool>;
  /**
   * Whether the agent is marked as a favorite
   */
  isFavorite: boolean;
  /**
   * Whether notifications for the agent are muted
   */
  isMuted: boolean;
  lastRunAt: DateTime | null;
  memories: Array<Memory>;
  /**
   * Default model to use
   */
  model: string;
  /**
   * The name of the Agent Definition
   */
  name: string;
  /**
   * The globally unique ID for the Agent Definition
   */
  nodeId: string;
  notificationsCount: number;
  owner: Identity;
  /**
   * Default field for safe mode. when set all nodes created will have to ask for confirmation from the user unless user overrides them manually
   */
  shouldAskForConfirmation: boolean;
  /**
   * This is the State Graph currently active on this Agent Definition.
   */
  stateGraph: StateGraph | null;
  /**
   * This is the State Graph Test Run currently active on this Agent Definition, notice that the State Graph set on this run will, most of the time, be different from the currently active State Graph of the Agent.
   */
  stateGraphTestRun: StateGraphTestRun | null;
  /**
   * The date the agent was last updated
   */
  updatedAt: DateTime;
};

export type AgentDefinitionConversationsConnection = {
  __typename?: 'AgentDefinitionConversationsConnection';
  edges: Array<AgentDefinitionConversationsConnectionEdge | null>;
  pageInfo: PageInfo;
  totalCount: number;
};

export type AgentDefinitionConversationsConnectionEdge = {
  __typename?: 'AgentDefinitionConversationsConnectionEdge';
  cursor: string;
  node: Conversation;
};

export type AgentDefinitionOrigin = 'CreateLindyModal' | 'Duplication' | 'InstallLindyAction' | 'Marketplace' | 'MarketplaceTemplatePack' | 'Occupation' | 'ShareLink' | 'SignUpTemplateIdUrlParameter' | 'UnknownTemplate' | '%future added value';

export type AgentFolder = {
  __typename?: 'AgentFolder';
  /**
   * The list of Agent Definitions inside this specific folder
   */
  agentDefinitions: Array<AgentDefinition>;
  clientId: string | null;
  id: string;
  /**
   * Whether the folder is expanded or not in the UI
   */
  isExpanded: boolean;
  /**
   * The name of the folder
   */
  name: string;
  /**
   * This is the total count of notifications across all agents inside this folder
   */
  notificationsCount: number;
  /**
   * This folder's parent folder's client ID. If it is `null` it means this is a root folder. The data structure is flattened, so this is not a recursive field. We use client ID instead of the folder's ID to prevent race conditions on folder creation since the is some latency in folder creation do to the required LLM call
   */
  parentClientId: string | null;
};

export type AgentFolderMovableItemInput = {
  __typename?: 'AgentFolderMovableItemInput';
  agentDefinitionId: string | null;
  agentFolderClientId: string | null;
};

export type AgentFolders = {
  __typename?: 'AgentFolders';
  /**
   * The list of agent folders. This is a flattened data structure, allowing us to have a "infinite" hierarchy of folders. The root folders have a `null` parent ID.
   */
  folders: Array<AgentFolder>;
  /**
   * This is the root "folder", which contains everything that is not inside any specific folder
   */
  root: AgentRootFolder;
};

export type AgentIcon = {
  __typename?: 'AgentIcon';
  color: AgentIconColor;
  name: AgentIconName;
};

export type AgentIconColor = 'Amber' | 'Blue' | 'DarkGrey' | 'Emerald' | 'Fuchsia' | 'Green' | 'Grey' | 'Indigo' | 'Lime' | 'Pink' | 'Purple' | 'Red' | 'Sky' | 'Slate' | 'Yellow' | '%future added value';

export type AgentIconInput = {
  __typename?: 'AgentIconInput';
  color: AgentIconColor;
  name: AgentIconName;
};

export type AgentIconName = 'AgentIcon1' | 'AgentIcon2' | 'AgentIcon3' | 'AgentIcon4' | 'AiStarsSparklesIcon' | 'AiThreeStarsSparklesIcon' | 'AirplaneIcon' | 'ArAugmentedRealityCardBoxDVirtualRealityVrIcon' | 'ArAugmentedRealityDViewCubeIcon' | 'ArchiveBoxIcon' | 'ArchiveBoxInboxFileIcon' | 'ArrowRightCircleIcon' | 'AtIcon' | 'AudioMusicPlaylistMusicalNoteIcon' | 'BagLuggageBuggageIcon' | 'BagShoppingIcon' | 'BankIcon' | 'BatteryChargingIcon' | 'BedIcon' | 'BellSimpleIcon' | 'BirthdayCakeIcon' | 'BookGuideInfoFaqIcon' | 'BookIcon' | 'BookmarkBannerFlagTagIcon' | 'BotIcon' | 'BrainAiThinkingIcon' | 'BrowserWindowAppDesktopIcon' | 'BrushColorIcon' | 'BubbleAnnotationMessageIcon' | 'BucketTrashCanIcon' | 'BugIssueIcon' | 'BuildingsIcon' | 'CalculatorIcon' | 'CalendarIcon' | 'ChartStatisticsGraphIcon' | 'ChatBubbleThoughtMessageIcon' | 'CheckRadioCircleCheckboxCheckCheckmarkConfirmIcon' | 'ClickIcon' | 'ClockCircleTimeIcon' | 'CloseXCircleRemoveIcon' | 'CloudIcon' | 'CloudySunCloudsIcon' | 'CodeBracketsIcon' | 'ColorIcon' | 'ColorSwatchPaletteColoursIcon' | 'ColorsPaletteColoursIcon' | 'CompassBrowserSafariWebInternetIcon' | 'CookiesIcon' | 'DashboardFastIcon' | 'DatabaseIcon' | 'DeleteRemoveGarbageWasteTrashCanIcon' | 'DiceFourIcon' | 'DiskSaveIcon' | 'EarthGlobeWorldIcon' | 'EditPencilPencilPenWriteDrawIcon' | 'EditSmallBoxPencilPenWriteDrawIcon' | 'EmailEnvelopeIcon' | 'EmailTriagerIcon' | 'ErrorWarningAlertIcon' | 'FileDocumentCloudSyncIcon' | 'FileDocumentsCopyIcon' | 'FingerPrintTouchIdIcon' | 'FireFlameHotHeatIcon' | 'FolderOpenFileIcon' | 'GamepadBaseRoundControllsGameJoystickIcon' | 'GasIcon' | 'GrowthGrowLeafsIcon' | 'HandFingerSelectIcon' | 'HeadphonesSupportIcon' | 'HeartLikeHealthLifeFavIcon' | 'HomeOpenHouseIcon' | 'ImacComputerDeviceIcon' | 'ImagesPhotosPicturesShotIcon' | 'InboxArchiveTrayShelfIcon' | 'InfoCircleTooltipIcon' | 'KeyIcon' | 'KeyboardMidiKeysPianoIcon' | 'KeyboardUpCloseDownOpenArrowIcon' | 'LabIcon' | 'LawLegalTermsImprintBalanceIcon' | 'LayersCopyIcon' | 'LifeBuoyHelpSupportIcon' | 'LightBulbIdeaLightIcon' | 'LinkChainIcon' | 'LocationExploreCompassIcon' | 'MacbookLaptopComputerDeviceIcon' | 'MagicBookMagicianIcon' | 'MagicHatIcon' | 'MagicStickIcon' | 'MeetingSchedulerIcon' | 'MinusCircleRemoveIcon' | 'MoonStarNightIcon' | 'MouseIcon' | 'NoteCardTextIcon' | 'NotificationBellActivityIcon' | 'OpenNoteBookPadIcon' | 'PauseIcon' | 'PeopleTogetherUserAvatarGroupIcon' | 'PhoneDeviceIphoneMobileIcon' | 'PhoneTelephoneContactIcon' | 'PictureImageFrameIcon' | 'PieChartGraphChartStatisticsIcon' | 'PiggyBankSaveMoneyIcon' | 'PinLocationBookmarkIcon' | 'PinLocationIcon' | 'PinLocationMapIcon' | 'PlayGoIcon' | 'PlayIcon' | 'PlusCircleAddIcon' | 'PoopSpamIcon' | 'PostcardCardNewsIcon' | 'PrinterPrintIcon' | 'QuestionmarkFaqHelpQuestionaireIcon' | 'RainbowCloudIcon' | 'ReadingListGlassesSteveJobsIcon' | 'ReloadRefreshRepeatIcon' | 'RocketStartupLaunchIcon' | 'ScriptFaxReceiptIcon' | 'SearchMagnifyingGlassIcon' | 'ServerDataStorageIcon' | 'ServerStorageDataCoinsMoneyIcon' | 'SettingsSliderThreeIcon' | 'ShakaCallMeHangTenIcon' | 'ShieldCheckSecurityProtectionIcon' | 'ShieldProtectSecurityCheckIcon' | 'ShieldSecurityProtectionIcon' | 'StarFavoriteAwardIcon' | 'StickyNoteIcon' | 'StorageHddSsdIcon' | 'SunLightModeDayIcon' | 'SunsetIcon' | 'TagSaleIcon' | 'TapeIcon' | 'TargetArrowGoalAimIcon' | 'TargetZoomIcon' | 'ToiletPaperWipeIcon' | 'TruckDeliveryIcon' | 'UmbrellaSecurityIcon' | 'VideoClapperboardIcon' | 'VoiceIcon' | 'VolumeFullSpeakerLoudSoundOnMusicIcon' | 'WebCryptoSpaceIcon' | 'ZapLightningFlashIcon' | '%future added value';

/**
 * An Agent Message is how agents can send or receive information.
 */
export type AgentMessage = Node & {
  __typename?: 'AgentMessage';
  id: string;
  /**
   * The globally unique ID for this resource
   */
  nodeId: string;
  owner: Identity;
};

export type AgentRequireConfirmation = 'Always' | 'Never' | 'Sometimes' | '%future added value';

export type AgentRootFolder = {
  __typename?: 'AgentRootFolder';
  /**
   * The list of Agent Definitions that are not inside any folder
   */
  agentDefinitions: Array<AgentDefinition>;
};

export type AgentStateNode = Node & StateGraphNode & {
  __typename?: 'AgentStateNode';
  /**
   * The actions of the agent state
   */
  actions: Array<StateGraphAction>;
  /**
   * The user-defined display name of the node
   */
  displayName: string | null;
  /**
   * The guidelines of the agent state
   */
  guidelines: string | null;
  /**
   * The unique identifier for the node
   */
  id: string;
  /**
   * Whether the node is manually positioned
   */
  manuallyPositioned: boolean;
  /**
   * The LLM model of the agent state
   */
  model: string;
  /**
   * The globally unique ID for the Agent State Node
   */
  nodeId: string;
  /**
   * The position of the node
   */
  position: NodePosition;
  /**
   * Whether the agent state requires confirmation
   */
  requireConfirmation: AgentRequireConfirmation;
  /**
   * The type of the node
   */
  type: NodeType;
};

/**
 * An Agent Template Definition contains information about the agent's behavior and can be added by users from the marketplace
 */
export type AgentTemplateDefinition = Node & {
  __typename?: 'AgentTemplateDefinition';
  category: TemplateCategory | null;
  /**
   * Whether or not this is an officially featured Lindy in the marketplace or not
   */
  featured: boolean;
  icon: AgentIcon | null;
  id: string;
  /**
   * The name of the Agent Template Definition
   */
  name: string;
  /**
   * The globally unique ID for the Agent Template Definition
   */
  nodeId: string;
  /**
   * The subtitle of the Agent Template Definition
   */
  subTitle: string | null;
};

export type Auth = Node & {
  __typename?: 'Auth';
  accountId: string;
  /**
   * The display name of the auth
   */
  displayName: string;
  /**
   * Schema for Custom Auth fields
   */
  fields: JSONObject | null;
  /**
   * Whether all required scopes are satisfied
   */
  hasAllScopes: boolean;
  id: string;
  /**
   * True if this auth is valid
   */
  isConnected: boolean;
  label: string;
  method: string;
  /**
   * The globally unique ID for the auth used by Relay
   */
  nodeId: string;
  owner: Identity;
  provider: string;
  redirectUri: URL | null;
  /**
   * The scopes of the auth
   */
  scopes: Array<string>;
  /**
   * The lindies that use this auth
   */
  usedBy: Array<AgentDefinition> | null;
};

export type AuthConfig = {
  method: AuthMethod;
  provider: string;
};

export type AuthMethod = 'Custom' | 'OAuth' | 'Token' | '%future added value';

export type AuthType = 'custom' | 'customoauth2' | 'oauth2' | 'token' | '%future added value';

export type BadRequestError = Error & {
  __typename?: 'BadRequestError';
  message: string;
};

export type BaseError = Error & {
  __typename?: 'BaseError';
  message: string;
};

export type BigNumber = unknown;

export type BillingVersion = 'V2' | 'V3' | '%future added value';

export type CancelIdentityEmailUpdateInput = {
  __typename?: 'CancelIdentityEmailUpdateInput';
  id: string;
};

export type CancelIdentityEmailUpdateOutput = {
  __typename?: 'CancelIdentityEmailUpdateOutput';
  identity: Identity;
};

export type ClearAgentNotificationsInput = {
  __typename?: 'ClearAgentNotificationsInput';
  /**
   * This is the ID of the agent
   */
  agentDefinitionId: string;
};

export type ClearAgentNotificationsOutput = {
  __typename?: 'ClearAgentNotificationsOutput';
  agent: AgentDefinition;
};

export type ClearConversationNotificationInput = {
  __typename?: 'ClearConversationNotificationInput';
  /**
   * This is the ID of the agent the conversation belongs to
   */
  agentDefinitionId: string;
  /**
   * This is the ID of the conversation
   */
  conversationId: string;
};

export type ClearConversationNotificationOutput = {
  __typename?: 'ClearConversationNotificationOutput';
  conversation: Conversation;
};

export type ContinueStateGraphTestRunWithTaskInput = {
  __typename?: 'ContinueStateGraphTestRunWithTaskInput';
  /**
   * This is the ID of the agent definition that is being tested
   */
  agentDefinitionId: string;
  /**
   * The list of attachments to add to this task.
   */
  attachmentIds: Array<string> | null;
  /**
   * This list of edges of the state graph
   */
  edges: Array<StateGraphEdgeInput>;
  /**
   * The list of nodes of the state graph. Keep in mind this is unrelated to Relay nodes.
   */
  nodes: Array<StateGraphNodeInput>;
  /**
   * The ID of the test run to be updated
   */
  stateGraphTestRunId: string | null;
  /**
   * The task to be sent as a follow-up
   */
  task: string;
};

export type ContinueStateGraphTestRunWithTaskOutput = {
  __typename?: 'ContinueStateGraphTestRunWithTaskOutput';
  agentDefinition: AgentDefinition;
  /**
   * This will contain the generated Agent Message (if any)
   */
  agentMessage: AgentMessage | null;
  stateGraphTestRun: StateGraphTestRun;
};

export type Conversation = Node & {
  __typename?: 'Conversation';
  /**
   * The AgentDefinition that this conversation belongs to
   */
  agentDefinition: AgentDefinition;
  blocks: ConversationBlocksConnection;
  createdAt: DateTime;
  /**
   * This is the node ID of the entry point node that was used when the conversation started. This will not change after the conversation has started, and it may be null till the conversation actually starts.
   */
  entryPointNodeId: string | null;
  /**
   * This is true if the conversation has new messages or actions that require the user's attention.
   */
  hasNotification: boolean;
  id: string;
  /**
   * The globally unique ID for the Conversation, this can be used to retrieve the Conversation using the node resolvers
   */
  nodeId: string;
  openChannels: Array<ConversationChannel>;
  owner: Identity;
  startingTriggerRunAttempt: TriggerRunAttempt | null;
  status: ConversationStatus;
  title: string | null;
  updatedAt: DateTime;
};

export type ConversationBlock = Node & {
  __typename?: 'ConversationBlock';
  content: ConversationBlockContentInterface;
  id: string;
  /**
   * The globally unique ID for the Block, this can be used to retrieve the Block using the node resolvers
   */
  nodeId: string;
  owner: Identity;
};

export type ConversationBlockContentBase = ConversationBlockContentInterface & {
  __typename?: 'ConversationBlockContentBase';
  /**
   * This is the full block object = This is a way to access the block's properties until all block types are added as separate GQL types.
   */
  asJSON: JSONObject;
  text: string;
  type: ConversationBlockType;
};

export type ConversationBlockContentInterface = {
  asJSON: JSONObject;
  text: string;
  type: ConversationBlockType;
};

export type ConversationBlockType = 'ACTION_CALL' | 'ACTION_NEEDED_API_TOKEN_AUTH' | 'ACTION_NEEDED_CUSTOM_AUTH' | 'ACTION_NEEDED_CUSTOM_OAUTH2' | 'ACTION_NEEDED_OAUTH2_AUTH' | 'ACTION_NEEDED_ONBOARDING' | 'ACTION_NEEDED_PAUSED' | 'ACTION_NEEDED_RAILS_ONBOARDING' | 'ACTION_RESULT' | 'ANSWER' | 'HINT' | 'INTERNAL_ACTION_CHECKPOINT' | 'INTERNAL_ACTION_INPUT_SUBMISSION' | 'INTERNAL_EXECUTION_DETAILS' | 'INTERNAL_EXECUTION_TRACE' | 'INTERNAL_FRAMEWORK_ERROR' | 'INTERNAL_GRAPH_TRAVERSAL' | 'INTERNAL_POSTHOG_SESSION_DETAILS' | 'KNOWLEDGE_BASE_SEARCH_RESULT' | 'MEDIA' | 'MESSAGE_RECEIVED' | 'MESSAGE_SENT' | 'SYSTEM_INSTRUCTION' | 'TALK' | 'UNSUPPORTED_CODE' | 'UNSUPPORTED_CODE_ERROR' | 'UNSUPPORTED_CODE_RESULT' | 'UNSUPPORTED_CODE_VARIABLES' | 'UNSUPPORTED_DECLARATION' | 'UNSUPPORTED_EXAMPLE' | 'UNSUPPORTED_INTERNAL_ACTION_NEEDED_CREDITS' | 'UNSUPPORTED_INTERNAL_CONVERSATION_PARAMS' | 'UNSUPPORTED_INTERNAL_EXECUTION_CHECKPOINT' | 'UNSUPPORTED_INTERNAL_GUIDELINES_UPDATED' | 'UNSUPPORTED_INTERNAL_INCOMPLETE_BLOCK' | 'UNSUPPORTED_INTERNAL_STREAMING' | 'UNSUPPORTED_MEMORY' | 'UNSUPPORTED_SYSTEM_PRUNING_HINT' | 'UNSUPPORTED_THOUGHT' | 'UNSUPPORTED_TYPESCRIPT' | 'UNSUPPORTED_USER' | '%future added value';

export type ConversationBlocksConnection = {
  __typename?: 'ConversationBlocksConnection';
  edges: Array<ConversationBlocksConnectionEdge | null>;
  pageInfo: PageInfo;
};

export type ConversationBlocksConnectionEdge = {
  __typename?: 'ConversationBlocksConnectionEdge';
  cursor: string;
  node: ConversationBlock;
};

export type ConversationChannel = {
  __typename?: 'ConversationChannel';
  /**
   * Which node created this channel (if any)
   */
  createdByNodeId: string | null;
  /**
   * The ID of the channel
   */
  id: string;
  /**
   * The instance key of the tool that is being used to manage the state of this channel
   */
  instanceKey: string | null;
  /**
   * If this returns null, it means this channel is not going to wake up the conversation on any node. Otherwise, it's the ID of the node that will be woken up.
   */
  startingNodeId: string | null;
  /**
   * The key of the tool that is being used to manage the state of this channel
   */
  statefulToolKey: string | null;
  tool: Tool;
  trigger: Trigger | null;
  type: string;
};

export type ConversationStarter = {
  __typename?: 'ConversationStarter';
  /**
   * The value of the conversation starter
   */
  value: string;
};

export type ConversationStatus = 'Blocked' | 'Cancelled' | 'Cancelling' | 'Completed' | 'Deleted' | 'Failed' | 'Pending' | 'PendingEnqueue' | 'PendingRestart' | 'Running' | '%future added value';

export type CopyAgentInput = {
  __typename?: 'CopyAgentInput';
  /**
   * This is the ID of the agent
   */
  agentDefinitionId: string;
  /**
   * This is the ID of the owner of the agent
   */
  id: string;
};

export type CopyAgentOutput = {
  __typename?: 'CopyAgentOutput';
  agent: AgentDefinition;
  owner: Identity;
};

export type CreateAgentFolderInput = {
  __typename?: 'CreateAgentFolderInput';
  /**
   * You can add agent definitions inside this folder while creating it to avoid having to call a separate mutation to do this
   */
  agentDefinitionIds: Array<string> | null;
  /**
   * A unique identifier for this folder that is generated by the client. This will link child folders to parent folders and is done with client ID rather than folder ID to prevent race conditions since folder creation has a significant delay due to the LLM calls.
   */
  clientId: string;
  /**
   * This is the ID of the owner of this resource (Identity ID)
   */
  id: string;
  /**
   * The name of the folder, this is optional, as it will be automatically generated from the list of agent definitions passed. The mutation will fail if this is not provided and the list of agents is empty.
   */
  name: string | null;
  /**
   * The client ID of the folder that this folder may be nested in. If null the folder will be at the root
   */
  parentClientId: string | null;
};

export type CreateAgentFolderOutput = {
  __typename?: 'CreateAgentFolderOutput';
  agentFolder: AgentFolder;
  owner: Identity;
};

export type CreateAgentInput = {
  __typename?: 'CreateAgentInput';
  /**
   * The ID of the folder to create the agent in
   */
  folderClientId: string | null;
  icon: AgentIconInput | null;
  /**
   * This is the ID of the owner of the agent
   */
  identityId: string;
  name: string | null;
};

export type CreateAgentOutput = {
  __typename?: 'CreateAgentOutput';
  agentDefinition: AgentDefinition;
  owner: Identity;
};

export type CreateStateGraphTestRunOutput = {
  __typename?: 'CreateStateGraphTestRunOutput';
  agentDefinition: AgentDefinition;
  /**
   * This will contain the generated Agent Message (if any)
   */
  agentMessage: AgentMessage | null;
  stateGraphTestRun: StateGraphTestRun;
};

export type CreateStateGraphTestRunWithActionInput = {
  __typename?: 'CreateStateGraphTestRunWithActionInput';
  /**
   * The ID of the action node to be tested
   */
  actionNodeId: string;
  /**
   * This is the ID of the agent definition that is being tested
   */
  agentDefinitionId: string;
  /**
   * This list of edges of the state graph
   */
  edges: Array<StateGraphEdgeInput>;
  /**
   * The list of nodes of the state graph. Keep in mind this is unrelated to Relay nodes.
   */
  nodes: Array<StateGraphNodeInput>;
};

export type CreateStateGraphTestRunWithActionOutput = {
  __typename?: 'CreateStateGraphTestRunWithActionOutput';
  agentDefinition: AgentDefinition;
  /**
   * This will contain the generated Agent Message (if any)
   */
  agentMessage: AgentMessage | null;
  stateGraphTestRun: StateGraphTestRun;
};

export type CreateStateGraphTestRunWithConversationInput = {
  __typename?: 'CreateStateGraphTestRunWithConversationInput';
  /**
   * The ID of the conversation from which the payload should be retrieved to start a new test run
   */
  conversationId: string;
};

export type CreateStateGraphTestRunWithConversationOutput = {
  __typename?: 'CreateStateGraphTestRunWithConversationOutput';
  agentDefinition: AgentDefinition;
  /**
   * This will contain the generated Agent Message (if any)
   */
  agentMessage: AgentMessage | null;
  stateGraphTestRun: StateGraphTestRun;
};

export type CreateStateGraphTestRunWithSyntheticTriggerPayloadInput = {
  __typename?: 'CreateStateGraphTestRunWithSyntheticTriggerPayloadInput';
  /**
   * This is the ID of the agent definition that is being tested
   */
  agentDefinitionId: string;
  /**
   * This list of edges of the state graph
   */
  edges: Array<StateGraphEdgeInput>;
  /**
   * The list of nodes of the state graph. Keep in mind this is unrelated to Relay nodes.
   */
  nodes: Array<StateGraphNodeInput>;
  /**
   * The ID of the Trigger Entry Point node to be tested
   */
  syntheticTriggerNodeId: string;
};

export type CreateStateGraphTestRunWithSyntheticTriggerPayloadOutput = {
  __typename?: 'CreateStateGraphTestRunWithSyntheticTriggerPayloadOutput';
  agentDefinition: AgentDefinition;
  /**
   * This will contain the generated Agent Message (if any)
   */
  agentMessage: AgentMessage | null;
  stateGraphTestRun: StateGraphTestRun;
};

export type CreateStateGraphTestRunWithTaskInput = {
  __typename?: 'CreateStateGraphTestRunWithTaskInput';
  /**
   * This is the ID of the agent definition that is being tested
   */
  agentDefinitionId: string;
  /**
   * The list of attachments to add to this task.
   */
  attachmentIds: Array<string> | null;
  /**
   * This list of edges of the state graph
   */
  edges: Array<StateGraphEdgeInput>;
  /**
   * The ID of the entry point node to be tested
   */
  entryPointNodeId: string;
  /**
   * The list of nodes of the state graph. Keep in mind this is unrelated to Relay nodes.
   */
  nodes: Array<StateGraphNodeInput>;
  /**
   * The task to be sent, this only works if the entry point being tested is a Conversation Entry Point
   */
  task: string;
};

export type CreditsInfo = {
  __typename?: 'CreditsInfo';
  consumed: number;
  creditAllocation: number;
  creditBalance: number;
  percentConsumed: number;
};

export type CustomAuthConfig = AuthConfig & {
  __typename?: 'CustomAuthConfig';
  /**
   * The fields of the auth configuration
   */
  fields: JSONObject;
  /**
   * The method of the auth configuration
   */
  method: AuthMethod;
  /**
   * The provider of the auth configuration
   */
  provider: string;
};

export type CustomOAuth2AuthConfig = AuthConfig & {
  __typename?: 'CustomOAuth2AuthConfig';
  /**
   * The fields of the auth configuration
   */
  fields: JSONObject;
  /**
   * The method of the auth configuration
   */
  method: AuthMethod;
  /**
   * The provider of the auth configuration
   */
  provider: string;
  /**
   * The scopes of the auth configuration
   */
  scopes: Array<string>;
};

export type Date = unknown;

export type DateTime = unknown;

export type DeleteAgentFolderInput = {
  __typename?: 'DeleteAgentFolderInput';
  agentFolderId: string;
  /**
   * This is the ID of the owner of the agent folder
   */
  id: string;
};

export type DeleteAgentFolderOutput = {
  __typename?: 'DeleteAgentFolderOutput';
  owner: Identity;
};

export type DeleteAgentInput = {
  __typename?: 'DeleteAgentInput';
  /**
   * This is the ID of the agent
   */
  agentDefinitionId: string;
  /**
   * This is the ID of the owner of the agent
   */
  id: string;
};

export type DeleteAgentOutput = {
  __typename?: 'DeleteAgentOutput';
  agent: AgentDefinition;
  owner: Identity;
};

export type DeleteAuthInput = {
  __typename?: 'DeleteAuthInput';
  /**
   * The ID of the auth
   */
  authId: string;
};

export type DeleteAuthOutput = {
  __typename?: 'DeleteAuthOutput';
  /**
   * The node ID of the deleted auth
   */
  deletedAuthNodeId: string;
};

export type DeleteConversationInput = {
  __typename?: 'DeleteConversationInput';
  conversationId: string;
};

export type DeleteConversationOutput = {
  __typename?: 'DeleteConversationOutput';
  conversation: Conversation;
};

export type DeleteIdentityInput = {
  __typename?: 'DeleteIdentityInput';
  id: string;
};

export type DeleteIdentityOutput = {
  __typename?: 'DeleteIdentityOutput';
  identity: Identity;
};

export type DeleteStateGraphTestRunInput = {
  __typename?: 'DeleteStateGraphTestRunInput';
  /**
   * The ID of the State Graph Test Run to be deleted
   */
  stateGraphTestRunId: string;
};

export type DeleteStateGraphTestRunOutput = {
  __typename?: 'DeleteStateGraphTestRunOutput';
  agentDefinition: AgentDefinition;
  /**
   * This is the node ID (globally unique ID, used by Relay) of the StateGraph test run that got removed
   */
  deletedStateGraphTestRunNodeId: string;
};

export type DynamicLoaderDetails = {
  __typename?: 'DynamicLoaderDetails';
  /**
   * The dependencies of the dynamic loader.
   */
  dependencies: Array<string>;
  /**
   * The description of the dynamic loader.
   */
  description: string | null;
  /**
   * The display name of the dynamic loader.
   */
  displayName: string;
  /**
   * The field of the dynamic loader.
   */
  field: string;
  icon: DynamicLoaderIcon;
  /**
   * The key of the dynamic loader.
   */
  key: string;
  /**
   * The placeholder of the dynamic loader.
   */
  placeholder: string | null;
};

export type DynamicLoaderIcon = 'GoogleSheetRow' | 'UNKNOWN' | 'UnknownIcon' | '%future added value';

export type DynamicLoaders = {
  __typename?: 'DynamicLoaders';
  loaders: Array<DynamicLoaderDetails>;
  /**
   * Whether to refetch the input after loading.
   */
  refetchInputAfterLoad: boolean;
};

export type EdgeType = 'Observe' | 'Standard' | '%future added value';

export type EntryNodeType = 'Conversation' | 'Trigger' | '%future added value';

export type EntryPointConversationNode = EntryPointNode & Node & StateGraphNode & {
  __typename?: 'EntryPointConversationNode';
  /**
   * The conversation starters of the entry point conversation node
   */
  conversationStarters: Array<ConversationStarter>;
  /**
   * The user-defined display name of the node
   */
  displayName: string | null;
  /**
   * The type of the entry node
   */
  entryType: EntryNodeType;
  /**
   * The greeting message of the entry point conversation node
   */
  greetingMessage: string | null;
  /**
   * The unique identifier for the node
   */
  id: string;
  /**
   * Whether the node is manually positioned
   */
  manuallyPositioned: boolean;
  /**
   * The globally unique ID for the Conversation Node
   */
  nodeId: string;
  /**
   * The position of the node
   */
  position: NodePosition;
  /**
   * The type of the node
   */
  type: NodeType;
};

export type EntryPointNode = {
  entryType: EntryNodeType;
};

export type EntryPointTriggerNode = EntryPointNode & Node & StateGraphNode & {
  __typename?: 'EntryPointTriggerNode';
  /**
   * The auth of the trigger
   */
  auth: Auth | null;
  /**
   * The auth entries available for the trigger
   */
  authList: Array<Auth>;
  /**
   * The user-defined display name of the node
   */
  displayName: string | null;
  /**
   * The type of the entry node
   */
  entryType: EntryNodeType;
  /**
   * The unique identifier for the node
   */
  id: string;
  /**
   * Whether the node is manually positioned
   */
  manuallyPositioned: boolean;
  /**
   * The globally unique ID for the Trigger Node
   */
  nodeId: string;
  /**
   * The position of the node
   */
  position: NodePosition;
  /**
   * The trigger database object of the entry point trigger node
   */
  trigger: Trigger | null;
  /**
   * The trigger definition of the entry point trigger node
   */
  triggerDefinition: TriggerDefinition | null;
  /**
   * The type of the node
   */
  type: NodeType;
};

export type Error = {
  message: string;
};

export type Identity = Node & {
  __typename?: 'Identity';
  agentDefinitions: Array<AgentDefinition>;
  /**
   * This represents the user's agent file structure.
   */
  agentFolders: AgentFolders;
  auth: Array<Auth>;
  /**
   * The avatar URL of the user
   */
  avatar: Image | null;
  billingInfo: SubscriptionInfo | null;
  /**
   * The date the user account was created
   */
  createdAt: Date;
  currentAgentDefinition: AgentDefinition | null;
  email: string;
  enabledFeatureFlags: Array<string>;
  id: string;
  /**
   * True if and only if this user is a Lindy admin
   */
  isAdmin: boolean;
  /**
   * Whether this user is considered a medical scribe customer, determined by the occupation associated with this user.
   */
  isMedicalScribe: boolean;
  name: string;
  /**
   * The globally unique ID for the user used by Relay
   */
  nodeId: string;
  owner: Identity;
  /**
   * If user has requested an email address change, this field contains the new unconfirmed email address. It is cleared by setting to null/empty string once user verifies the new email address through the confirmation link sent to that address.
   */
  pendingEmail: string | null;
  phone: PhoneNumber | null;
};

export type Image = {
  __typename?: 'Image';
  /**
   * The height of the image in pixels
   */
  heightPx: number | null;
  /**
   * The URL of the image
   */
  url: string;
  /**
   * The width of the image in pixels
   */
  widthPx: number | null;
};

export type InstallAgentTemplateInput = {
  __typename?: 'InstallAgentTemplateInput';
  /**
   * This is the ID of the owner of the agent
   */
  identityId: string;
  /**
   * The origin of the agent template
   */
  origin: AgentDefinitionOrigin | null;
  /**
   * The ID of the template to create the agent from
   */
  templateId: string;
};

export type InstallAgentTemplateOutput = {
  __typename?: 'InstallAgentTemplateOutput';
  agentDefinition: AgentDefinition;
  owner: Identity;
};

export type JSON = unknown;

export type JSONObject = unknown;

export type JSONSchema = unknown;

export type KnowledgeBaseSource = 'Box' | 'Confluence' | 'Dropbox' | 'Freshdesk' | 'GitHub' | 'Gitbook' | 'Gmail' | 'GoogleDrive' | 'Intercom' | 'Notion' | 'NotionDatabase' | 'OneDrive' | 'Outlook' | 'RawText' | 'RssFeed' | 'S3' | 'Salesforce' | 'SharePoint' | 'UNKNOWN' | 'Unknown' | 'UploadedFile' | 'WebScrape' | 'Zendesk' | 'Zotero' | '%future added value';

export type KnowledgeBaseSyncStatus = 'CrawlingWebsite' | 'Delayed' | 'EvaluatingResync' | 'QueueForSync' | 'QueuedForOcr' | 'RateLimited' | 'Ready' | 'SyncAborted' | 'SyncError' | 'Syncing' | 'UNKNOWN' | 'Unknown' | 'Uploading' | '%future added value';

export type KnowledgeSource = {
  __typename?: 'KnowledgeSource';
  /**
   * The number of entries in the knowledge source
   */
  entryCount: number;
  /**
   * Whether the knowledge source is enabled
   */
  isEnabled: boolean;
  /**
   * The type of the member
   */
  source: KnowledgeBaseSource;
  /**
   * The sync status of the knowledge source
   */
  syncStatus: KnowledgeBaseSyncStatus;
  /**
   * The date when the sync will be completed
   */
  syncToBeCompletedBy: Date | null;
  /**
   * The total file size of the knowledge source
   */
  totalFileSize: number;
  /**
   * The total number of characters in the knowledge source
   */
  totalNumCharacters: number;
  /**
   * The total number of embeddings in the knowledge source
   */
  totalNumEmbeddings: number;
  /**
   * The total number of errors in the knowledge source
   */
  totalNumErrors: number;
  /**
   * The total number of tokens in the knowledge source
   */
  totalNumTokens: number;
};

export type LindyTool = 'ActionNetwork' | 'ActiveCampaign' | 'Adalo' | 'Affinity' | 'AgileCrm' | 'Airtable' | 'AmazonS3' | 'ApiTemplateIo' | 'Apitable' | 'Approval' | 'Asana' | 'Automizy' | 'Autopilot' | 'AwsCertificateManager' | 'AwsComprehend' | 'AwsDynamoDB' | 'AwsElb' | 'AwsRekognition' | 'AwsS3' | 'AwsSes' | 'AwsTranscribe' | 'BambooHr' | 'Bannerbear' | 'Baserow' | 'Beeminder' | 'Binance' | 'Bitly' | 'Bitwarden' | 'Box' | 'Brevo' | 'BrightData' | 'BrowseAI' | 'Browser' | 'Bubble' | 'CalCom' | 'Calendar' | 'CalendarEvent' | 'Calendly' | 'Certopus' | 'Channels' | 'Chargebee' | 'Chatbots' | 'CircleCi' | 'CiscoWebex' | 'CitrixAdc' | 'Clarifai' | 'Clearbit' | 'Clickup' | 'Clockify' | 'Clockodo' | 'Cloudflare' | 'Cockpit' | 'Coda' | 'CoinGecko' | 'Contentful' | 'Contiguity' | 'ConvertKit' | 'Copper' | 'Cortex' | 'CrowdDev' | 'Csv' | 'CustomerIo' | 'DataMapper' | 'DateHelper' | 'Debug' | 'DebugInstance' | 'DeepL' | 'Deepl' | 'Delay' | 'Demio' | 'Dhl' | 'Directory' | 'DirectoryEntity' | 'Discord' | 'Discourse' | 'Disqus' | 'DocumentReader' | 'DraftEmail' | 'Drift' | 'Drip' | 'Dropbox' | 'Dropcontact' | 'ERPNext' | 'Egoi' | 'ElasticSecurity' | 'Elasticsearch' | 'EmailSend' | 'Emelia' | 'FacebookLeads' | 'FacebookPages' | 'Figma' | 'FileEntity' | 'FileHandler' | 'FileHelper' | 'Flow' | 'Freshdesk' | 'Freshsales' | 'Freshservice' | 'FreshworksCrm' | 'GSuiteAdmin' | 'GcloudPubsub' | 'GetResponse' | 'Ghost' | 'Ghostcms' | 'GitHub' | 'GitHubPullRequest' | 'GitRepository' | 'Github' | 'Gitlab' | 'Gmail' | 'GmailMessage' | 'GoToWebinar' | 'Google' | 'GoogleAds' | 'GoogleAnalytics' | 'GoogleBigQuery' | 'GoogleBooks' | 'GoogleChat' | 'GoogleCloudNaturalLanguage' | 'GoogleCloudStorage' | 'GoogleContacts' | 'GoogleDocs' | 'GoogleDrive' | 'GoogleFirebaseCloudFirestore' | 'GoogleForms' | 'GoogleMyBusiness' | 'GooglePeopleContacts' | 'GoogleSearch' | 'GoogleSheets' | 'GoogleSlides' | 'GoogleTasks' | 'GoogleTranslate' | 'Gotify' | 'Grafana' | 'Gravityforms' | 'HackerNews' | 'HaloPSA' | 'Harvest' | 'HelpScout' | 'HighLevel' | 'HomeAssistant' | 'Http' | 'Hubspot' | 'HumanticAi' | 'Imap' | 'Inbox' | 'InstagramBusiness' | 'Intercom' | 'InvoiceNinja' | 'ItemLists' | 'Iterable' | 'Jenkins' | 'Jira' | 'Jotform' | 'Keap' | 'Kimai' | 'Kitemaker' | 'KizeoForms' | 'KnowledgeBase' | 'KoBoToolbox' | 'Lemlist' | 'Lindy' | 'LindyAgent' | 'LindyAttachment' | 'LindyDatabaseExperimental' | 'LindyEmbed' | 'LindyMail' | 'LindyMailMessage' | 'LindyMedia' | 'LindyMeeting' | 'LindyMessenger' | 'LindyStateGraphBuilder' | 'LindyWebhook' | 'Linear' | 'LinkedIn' | 'ListSearch' | 'Llmrails' | 'Localai' | 'LoneScale' | 'Magento2' | 'Mailcheck' | 'Mailchimp' | 'MailerLite' | 'Mandrill' | 'Marketstack' | 'Mastodon' | 'MathHelper' | 'Matrix' | 'Mattermost' | 'Mautic' | 'Medium' | 'MessageBird' | 'Metabase' | 'MicrosoftDynamicsCrm' | 'MicrosoftExcel' | 'MicrosoftExcel365' | 'MicrosoftGraphSecurity' | 'MicrosoftOneDrive' | 'MicrosoftOnedrive' | 'MicrosoftOutlook' | 'MicrosoftOutlookMessage' | 'MicrosoftTeams' | 'MicrosoftToDo' | 'Mindee' | 'Misp' | 'Mocean' | 'Monday' | 'MondayCom' | 'MonicaCrm' | 'Msg91' | 'Mysql' | 'Nasa' | 'NaturalLanguage' | 'Netlify' | 'NextCloud' | 'Nifty' | 'NocoDB' | 'Notion' | 'Npm' | 'Ntfy' | 'Odoo' | 'Onboarding' | 'OnboardingV3' | 'OneSimpleApi' | 'Onfleet' | 'OpenAI' | 'OpenRouter' | 'Orbit' | 'Oura' | 'Paddle' | 'PagerDuty' | 'Pastebin' | 'Pastefy' | 'PayPal' | 'PeopleDataLabs' | 'Phantombuster' | 'PhilipsHue' | 'Pipedrive' | 'Plivo' | 'PostBin' | 'PostHog' | 'Postgres' | 'Preferences' | 'ProfitWell' | 'Pushbullet' | 'Pushcut' | 'Pushover' | 'Qdrant' | 'QuickBase' | 'QuickBooks' | 'Raindrop' | 'Resend' | 'Rocketchat' | 'Rss' | 'Rundeck' | 'S3' | 'Saastic' | 'Salesforce' | 'Salesmate' | 'Schedule' | 'ScriptTool' | 'SeaTable' | 'SecurityScorecard' | 'Segment' | 'SendGrid' | 'SendToChannel' | 'Sendfox' | 'Sendgrid' | 'Sendinblue' | 'Sendy' | 'SentryIo' | 'ServiceNow' | 'Sftp' | 'Shopify' | 'Signl4' | 'Simplepdf' | 'Slack' | 'SlackMessage' | 'Sms77' | 'Smtp' | 'Soap' | 'Splunk' | 'Spontit' | 'Spotify' | 'Square' | 'Ssh' | 'StabilityAi' | 'State' | 'Store' | 'Storyblok' | 'Strapi' | 'Strava' | 'Stripe' | 'Supabase' | 'Surveymonkey' | 'SyncroMsp' | 'Tags' | 'Taiga' | 'Talkable' | 'Tally' | 'Tapfiliate' | 'Task' | 'Telegram' | 'TelegramBot' | 'Test' | 'TextGeneration' | 'TextHelper' | 'TheHiveProject' | 'Tidycal' | 'Timer' | 'Todoist' | 'TravisCi' | 'Trello' | 'Twake' | 'Twilio' | 'Twist' | 'Twitter' | 'Typeform' | 'UI' | 'UnleashedSoftware' | 'Uplead' | 'UptimeRobot' | 'UrlScanIo' | 'VenafiTlsProtectCloud' | 'VenafiTlsProtectDatacenter' | 'Vero' | 'Vonage' | 'Vtex' | 'Wandb' | 'Webflow' | 'Wekan' | 'WhatsApp' | 'Wise' | 'WooCommerce' | 'Wordpress' | 'Xero' | 'Xml' | 'YouTube' | 'Yourls' | 'Zammad' | 'Zendesk' | 'ZohoCrm' | 'ZohoInvoice' | 'Zoom' | 'Zulip' | '%future added value';

export type LogicNode = Node & StateGraphNode & {
  __typename?: 'LogicNode';
  /**
   * The user-defined display name of the node
   */
  displayName: string | null;
  /**
   * The unique identifier for the node
   */
  id: string;
  /**
   * Whether the node is manually positioned
   */
  manuallyPositioned: boolean;
  /**
   * The LLM model of the logic node
   */
  model: string;
  /**
   * The globally unique ID for the Logic Node
   */
  nodeId: string;
  /**
   * The position of the node
   */
  position: NodePosition;
  /**
   * The type of the node
   */
  type: NodeType;
};

export type Memory = {
  __typename?: 'Memory';
  id: string | null;
  isActive: boolean;
  value: string;
};

export type MemoryInput = {
  __typename?: 'MemoryInput';
  id: string | null;
  /**
   * Determines whether this memory will be included into the prompt or not. Defaulting to true
   */
  isActive: boolean | null;
  value: string;
};

export type MoveItemsToAgentFolderInput = {
  __typename?: 'MoveItemsToAgentFolderInput';
  /**
   * Setting this to null means "Move this to the root, so it has no parent folder".
   */
  agentFolderClientId: string | null;
  /**
   * The ID of the identity whose folder items are being moved
   */
  id: string;
  items: Array<AgentFolderMovableItemInput>;
};

export type MoveItemsToAgentFolderOutput = {
  __typename?: 'MoveItemsToAgentFolderOutput';
  owner: Identity;
};

export type Mutation = {
  __typename?: 'Mutation';
  cancelIdentityEmailUpdate: MutationCancelIdentityEmailUpdateResult;
  clearAgentNotifications: MutationClearAgentNotificationsResult;
  clearConversationNotification: MutationClearConversationNotificationResult;
  /**
   * This continues a State Graph Test Run through a follow-up task.
   * This receives no ID parameter as it will always update the active test run on the given Agent Definition.
   * Keep in mind that due to the immutable nature of State Graphs, this creates a new state graph and updates its reference in the Test Run document.
   * If the State Graph is invalid, a StateGraphValidationError will be returned.
   */
  continueStateGraphTestRunWithTask: MutationContinueStateGraphTestRunWithTaskResult;
  copyAgent: MutationCopyAgentResult;
  createAgent: MutationCreateAgentResult;
  createAgentFolder: MutationCreateAgentFolderResult;
  /**
   * Create a new State Graph Test Run from a given action node. This will always completely disable the previous conversation test run of the given agent definition (if any).
   */
  createStateGraphTestRunWithAction: MutationCreateStateGraphTestRunWithActionResult;
  /**
   * Create a new State Graph Test Run from an existing conversation.
   * This will grab the payload initially sent to start the conversation, and use it to create a _new_ conversation
   * with the same payload, and then immediately start it. This is going to use the currently active state graph.
   * This will always completely disable the previous conversation test run of the given agent definition (if any).
   */
  createStateGraphTestRunWithConversation: MutationCreateStateGraphTestRunWithConversationResult;
  /**
   * Create a new State Graph Test Run from a Trigger Entry Point node that supports synthetic payloads.
   * This will always completely disable the previous conversation test run of the given agent definition (if any).
   */
  createStateGraphTestRunWithSyntheticTriggerPayload: MutationCreateStateGraphTestRunWithSyntheticTriggerPayloadResult;
  /**
   * Create a new State Graph Test Run from a given Conversation entry point node. This will always completely disable the previous conversation test run of the given agent definition (if any).
   */
  createStateGraphTestRunWithTask: MutationCreateStateGraphTestRunWithTaskResult;
  deleteAgent: MutationDeleteAgentResult;
  deleteAgentFolder: MutationDeleteAgentFolderResult;
  deleteAuth: MutationDeleteAuthResult;
  deleteConversation: MutationDeleteConversationResult;
  deleteIdentity: MutationDeleteIdentityResult;
  /**
   * This will delete the given State Graph Test Run and ensure the triggers on the previously created conversation are disabled.
   */
  deleteStateGraphTestRunInput: MutationDeleteStateGraphTestRunInputResult;
  installAgentTemplate: MutationInstallAgentTemplateResult;
  moveItemsToAgentFolder: MutationMoveItemsToAgentFolderResult;
  /**
   * Reenables a live capture trigger
   */
  reenableLiveCaptureTrigger: Trigger;
  renameConversation: MutationRenameConversationResult;
  replaceAgentMemories: MutationReplaceAgentMemoriesResult;
  requestIdentityEmailUpdate: MutationRequestIdentityEmailUpdateResult;
  resendIdentityEmailUpdate: MutationResendIdentityEmailUpdateResult;
  /**
   * This retries a State Graph Test Run from an execution ID with the updated State Graph.
   * This receives no ID parameter as it will always update the active test run on the given Agent Definition.
   * Keep in mind that due to the immutable nature of State Graphs, this creates a new state graph and updates its reference in the Test Run document.
   * If the State Graph is invalid, a StateGraphValidationError will be returned.
   */
  retryStateGraphTestRunExecution: MutationRetryStateGraphTestRunExecutionResult;
  /**
   * Initializes a State Graph Test Run through a Trigger Entry Point and a Trigger Run Attempt.
   */
  selectStateGraphTestRunTriggerEntryPointPayload: MutationSelectStateGraphTestRunTriggerEntryPointPayloadResult;
  skipWakeUpTaskTrigger: MutationSkipWakeUpTaskTriggerResult;
  startConversationFromNode: MutationStartConversationFromNodeResult;
  /**
   * This initializes a State Graph Test Run with a Live Capture trigger.
   */
  startLiveCaptureForStateGraphTestRun: MutationStartLiveCaptureForStateGraphTestRunResult;
  /**
   * This ends the current Live Capture session for a State Graph Test Run.
   */
  stopLiveCaptureForStateGraphTestRun: MutationStopLiveCaptureForStateGraphTestRunResult;
  updateAgent: MutationUpdateAgentResult;
  updateAgentFolder: MutationUpdateAgentFolderResult;
  updateCurrentAgentDefinition: MutationUpdateCurrentAgentDefinitionResult;
  updateIdentity: MutationUpdateIdentityResult;
  updateIdentityAvatar: MutationUpdateIdentityAvatarResult;
  updateIdentityPassword: MutationUpdateIdentityPasswordResult;
  updateOrCreateCustomAuth: MutationUpdateOrCreateCustomAuthResult;
  updateOrCreateTokenAuth: MutationUpdateOrCreateTokenAuthResult;
};

export type MutationCancelIdentityEmailUpdateResult = BaseError | MutationCancelIdentityEmailUpdateSuccess | NotFoundError | UnauthorizedError | ValidationError;

export type MutationCancelIdentityEmailUpdateSuccess = {
  __typename?: 'MutationCancelIdentityEmailUpdateSuccess';
  data: CancelIdentityEmailUpdateOutput;
};

export type MutationClearAgentNotificationsResult = BaseError | MutationClearAgentNotificationsSuccess | NotFoundError | UnauthorizedError | ValidationError;

export type MutationClearAgentNotificationsSuccess = {
  __typename?: 'MutationClearAgentNotificationsSuccess';
  data: ClearAgentNotificationsOutput;
};

export type MutationClearConversationNotificationResult = BaseError | MutationClearConversationNotificationSuccess | NotFoundError | UnauthorizedError | ValidationError;

export type MutationClearConversationNotificationSuccess = {
  __typename?: 'MutationClearConversationNotificationSuccess';
  data: ClearConversationNotificationOutput;
};

export type MutationContinueStateGraphTestRunWithTaskResult = BaseError | MutationContinueStateGraphTestRunWithTaskSuccess | NotFoundError | StateGraphValidationError | UnauthorizedError | ValidationError;

export type MutationContinueStateGraphTestRunWithTaskSuccess = {
  __typename?: 'MutationContinueStateGraphTestRunWithTaskSuccess';
  data: ContinueStateGraphTestRunWithTaskOutput;
};

export type MutationCopyAgentResult = BaseError | MutationCopyAgentSuccess | NotFoundError | UnauthorizedError | ValidationError;

export type MutationCopyAgentSuccess = {
  __typename?: 'MutationCopyAgentSuccess';
  data: CopyAgentOutput;
};

export type MutationCreateAgentFolderResult = BaseError | MutationCreateAgentFolderSuccess | NotFoundError | UnauthorizedError | ValidationError;

export type MutationCreateAgentFolderSuccess = {
  __typename?: 'MutationCreateAgentFolderSuccess';
  data: CreateAgentFolderOutput;
};

export type MutationCreateAgentResult = BaseError | MutationCreateAgentSuccess | NotFoundError | UnauthorizedError | ValidationError;

export type MutationCreateAgentSuccess = {
  __typename?: 'MutationCreateAgentSuccess';
  data: CreateAgentOutput;
};

export type MutationCreateStateGraphTestRunWithActionResult = BaseError | MutationCreateStateGraphTestRunWithActionSuccess | NotFoundError | StateGraphValidationError | UnauthorizedError | ValidationError;

export type MutationCreateStateGraphTestRunWithActionSuccess = {
  __typename?: 'MutationCreateStateGraphTestRunWithActionSuccess';
  data: CreateStateGraphTestRunWithActionOutput;
};

export type MutationCreateStateGraphTestRunWithConversationResult = BaseError | MutationCreateStateGraphTestRunWithConversationSuccess | NotFoundError | StateGraphValidationError | UnauthorizedError | ValidationError;

export type MutationCreateStateGraphTestRunWithConversationSuccess = {
  __typename?: 'MutationCreateStateGraphTestRunWithConversationSuccess';
  data: CreateStateGraphTestRunWithConversationOutput;
};

export type MutationCreateStateGraphTestRunWithSyntheticTriggerPayloadResult = BaseError | MutationCreateStateGraphTestRunWithSyntheticTriggerPayloadSuccess | NotFoundError | StateGraphValidationError | UnauthorizedError | ValidationError;

export type MutationCreateStateGraphTestRunWithSyntheticTriggerPayloadSuccess = {
  __typename?: 'MutationCreateStateGraphTestRunWithSyntheticTriggerPayloadSuccess';
  data: CreateStateGraphTestRunWithSyntheticTriggerPayloadOutput;
};

export type MutationCreateStateGraphTestRunWithTaskResult = BaseError | MutationCreateStateGraphTestRunWithTaskSuccess | NotFoundError | StateGraphValidationError | UnauthorizedError | ValidationError;

export type MutationCreateStateGraphTestRunWithTaskSuccess = {
  __typename?: 'MutationCreateStateGraphTestRunWithTaskSuccess';
  data: CreateStateGraphTestRunOutput;
};

export type MutationDeleteAgentFolderResult = BaseError | MutationDeleteAgentFolderSuccess | NotFoundError | UnauthorizedError | ValidationError;

export type MutationDeleteAgentFolderSuccess = {
  __typename?: 'MutationDeleteAgentFolderSuccess';
  data: DeleteAgentFolderOutput;
};

export type MutationDeleteAgentResult = BaseError | MutationDeleteAgentSuccess | NotFoundError | UnauthorizedError | ValidationError;

export type MutationDeleteAgentSuccess = {
  __typename?: 'MutationDeleteAgentSuccess';
  data: DeleteAgentOutput;
};

export type MutationDeleteAuthResult = BaseError | MutationDeleteAuthSuccess | NotFoundError | UnauthorizedError | ValidationError;

export type MutationDeleteAuthSuccess = {
  __typename?: 'MutationDeleteAuthSuccess';
  data: DeleteAuthOutput;
};

export type MutationDeleteConversationResult = BaseError | MutationDeleteConversationSuccess | NotFoundError | UnauthorizedError | ValidationError;

export type MutationDeleteConversationSuccess = {
  __typename?: 'MutationDeleteConversationSuccess';
  data: DeleteConversationOutput;
};

export type MutationDeleteIdentityResult = BaseError | MutationDeleteIdentitySuccess | NotFoundError | UnauthorizedError | ValidationError;

export type MutationDeleteIdentitySuccess = {
  __typename?: 'MutationDeleteIdentitySuccess';
  data: DeleteIdentityOutput;
};

export type MutationDeleteStateGraphTestRunInputResult = BaseError | MutationDeleteStateGraphTestRunInputSuccess | NotFoundError | UnauthorizedError | ValidationError;

export type MutationDeleteStateGraphTestRunInputSuccess = {
  __typename?: 'MutationDeleteStateGraphTestRunInputSuccess';
  data: DeleteStateGraphTestRunOutput;
};

export type MutationInstallAgentTemplateResult = BaseError | MutationInstallAgentTemplateSuccess | NotFoundError | UnauthorizedError | ValidationError;

export type MutationInstallAgentTemplateSuccess = {
  __typename?: 'MutationInstallAgentTemplateSuccess';
  data: InstallAgentTemplateOutput;
};

export type MutationMoveItemsToAgentFolderResult = BaseError | MutationMoveItemsToAgentFolderSuccess | NotFoundError | UnauthorizedError | ValidationError;

export type MutationMoveItemsToAgentFolderSuccess = {
  __typename?: 'MutationMoveItemsToAgentFolderSuccess';
  data: MoveItemsToAgentFolderOutput;
};

export type MutationRenameConversationResult = BaseError | MutationRenameConversationSuccess | NotFoundError | UnauthorizedError | ValidationError;

export type MutationRenameConversationSuccess = {
  __typename?: 'MutationRenameConversationSuccess';
  data: RenameConversationOutput;
};

export type MutationReplaceAgentMemoriesResult = BaseError | MutationReplaceAgentMemoriesSuccess | NotFoundError | UnauthorizedError | ValidationError;

export type MutationReplaceAgentMemoriesSuccess = {
  __typename?: 'MutationReplaceAgentMemoriesSuccess';
  data: ReplaceAgentMemoryOutput;
};

export type MutationRequestIdentityEmailUpdateResult = BaseError | MutationRequestIdentityEmailUpdateSuccess | NotFoundError | UnauthorizedError | ValidationError;

export type MutationRequestIdentityEmailUpdateSuccess = {
  __typename?: 'MutationRequestIdentityEmailUpdateSuccess';
  data: RequestIdentityEmailUpdateOutput;
};

export type MutationResendIdentityEmailUpdateResult = BaseError | MutationResendIdentityEmailUpdateSuccess | NotFoundError | UnauthorizedError | ValidationError;

export type MutationResendIdentityEmailUpdateSuccess = {
  __typename?: 'MutationResendIdentityEmailUpdateSuccess';
  data: ResendIdentityEmailUpdateOutput;
};

export type MutationRetryStateGraphTestRunExecutionResult = BaseError | MutationRetryStateGraphTestRunExecutionSuccess | NotFoundError | StateGraphValidationError | UnauthorizedError | ValidationError;

export type MutationRetryStateGraphTestRunExecutionSuccess = {
  __typename?: 'MutationRetryStateGraphTestRunExecutionSuccess';
  data: RetryStateGraphTestRunExecutionOutput;
};

export type MutationSelectStateGraphTestRunTriggerEntryPointPayloadResult = BaseError | MutationSelectStateGraphTestRunTriggerEntryPointPayloadSuccess | NotFoundError | StateGraphValidationError | UnauthorizedError | ValidationError;

export type MutationSelectStateGraphTestRunTriggerEntryPointPayloadSuccess = {
  __typename?: 'MutationSelectStateGraphTestRunTriggerEntryPointPayloadSuccess';
  data: SelectStateGraphTestRunTriggerEntryPointPayloadOutput;
};

export type MutationSkipWakeUpTaskTriggerResult = BaseError | MutationSkipWakeUpTaskTriggerSuccess | NotFoundError | UnauthorizedError | ValidationError;

export type MutationSkipWakeUpTaskTriggerSuccess = {
  __typename?: 'MutationSkipWakeUpTaskTriggerSuccess';
  data: SkipWakeUpTaskTriggerOutput;
};

export type MutationStartConversationFromNodeResult = BaseError | MutationStartConversationFromNodeSuccess | NotFoundError | UnauthorizedError | ValidationError;

export type MutationStartConversationFromNodeSuccess = {
  __typename?: 'MutationStartConversationFromNodeSuccess';
  data: StartConversationFromNodeOutput;
};

export type MutationStartLiveCaptureForStateGraphTestRunResult = BaseError | MutationStartLiveCaptureForStateGraphTestRunSuccess | NotFoundError | StateGraphValidationError | UnauthorizedError | ValidationError;

export type MutationStartLiveCaptureForStateGraphTestRunSuccess = {
  __typename?: 'MutationStartLiveCaptureForStateGraphTestRunSuccess';
  data: StartLiveCaptureForStateGraphTestRunOutput;
};

export type MutationStopLiveCaptureForStateGraphTestRunResult = BaseError | MutationStopLiveCaptureForStateGraphTestRunSuccess | NotFoundError | StateGraphValidationError | UnauthorizedError | ValidationError;

export type MutationStopLiveCaptureForStateGraphTestRunSuccess = {
  __typename?: 'MutationStopLiveCaptureForStateGraphTestRunSuccess';
  data: StopLiveCaptureForStateGraphTestRunOutput;
};

export type MutationUpdateAgentFolderResult = BaseError | MutationUpdateAgentFolderSuccess | NotFoundError | UnauthorizedError | ValidationError;

export type MutationUpdateAgentFolderSuccess = {
  __typename?: 'MutationUpdateAgentFolderSuccess';
  data: UpdateAgentFolderOutput;
};

export type MutationUpdateAgentResult = BaseError | MutationUpdateAgentSuccess | NotFoundError | UnauthorizedError | ValidationError;

export type MutationUpdateAgentSuccess = {
  __typename?: 'MutationUpdateAgentSuccess';
  data: UpdateAgentOutput;
};

export type MutationUpdateCurrentAgentDefinitionResult = BaseError | MutationUpdateCurrentAgentDefinitionSuccess | NotFoundError | UnauthorizedError | ValidationError;

export type MutationUpdateCurrentAgentDefinitionSuccess = {
  __typename?: 'MutationUpdateCurrentAgentDefinitionSuccess';
  data: UpdateCurrentAgentDefinitionOutput;
};

export type MutationUpdateIdentityAvatarResult = BaseError | MutationUpdateIdentityAvatarSuccess | NotFoundError | UnauthorizedError | ValidationError;

export type MutationUpdateIdentityAvatarSuccess = {
  __typename?: 'MutationUpdateIdentityAvatarSuccess';
  data: UpdateIdentityAvatarOutput;
};

export type MutationUpdateIdentityPasswordResult = BaseError | MutationUpdateIdentityPasswordSuccess | NotFoundError | UnauthorizedError | ValidationError;

export type MutationUpdateIdentityPasswordSuccess = {
  __typename?: 'MutationUpdateIdentityPasswordSuccess';
  data: UpdateIdentityPasswordOutput;
};

export type MutationUpdateIdentityResult = BaseError | MutationUpdateIdentitySuccess | NotFoundError | UnauthorizedError | ValidationError;

export type MutationUpdateIdentitySuccess = {
  __typename?: 'MutationUpdateIdentitySuccess';
  data: UpdateIdentityOutput;
};

export type MutationUpdateOrCreateCustomAuthResult = BaseError | MutationUpdateOrCreateCustomAuthSuccess | NotFoundError | UnauthorizedError | ValidationError;

export type MutationUpdateOrCreateCustomAuthSuccess = {
  __typename?: 'MutationUpdateOrCreateCustomAuthSuccess';
  data: UpdateOrCreateCustomAuthOutput;
};

export type MutationUpdateOrCreateTokenAuthResult = BaseError | MutationUpdateOrCreateTokenAuthSuccess | NotFoundError | UnauthorizedError | ValidationError;

export type MutationUpdateOrCreateTokenAuthSuccess = {
  __typename?: 'MutationUpdateOrCreateTokenAuthSuccess';
  data: UpdateOrCreateTokenAuthOutput;
};

export type Node = {
  nodeId: string;
};

export type NodePosition = {
  __typename?: 'NodePosition';
  x: number;
  y: number;
};

export type NodeType = 'Action' | 'AgentState' | 'EntryPoint' | 'Logic' | '%future added value';

export type NotFoundError = Error & {
  __typename?: 'NotFoundError';
  entity: string;
  message: string;
};

export type OAuth2AuthConfig = AuthConfig & {
  __typename?: 'OAuth2AuthConfig';
  /**
   * The method of the auth configuration
   */
  method: AuthMethod;
  /**
   * The provider of the auth configuration
   */
  provider: string;
  /**
   * The scopes of the auth configuration
   */
  scopes: Array<string>;
};

export type ObservableChannel = {
  __typename?: 'ObservableChannel';
  /**
   * The stateful tool key of the observable channel
   */
  statefulToolKey: string;
};

export type ObservableChannelMetadata = {
  __typename?: 'ObservableChannelMetadata';
  /**
   * The schema of messages received through this channel
   */
  agentMessageReceivedSchema: JSONObject;
  /**
   * The user facing label for the edge that executes immediately after the action creating this channel.
   */
  immediateEdgeLabel: string;
  /**
   * The user facing label for the edge that executes after this channel receives a message.
   */
  observableEdgeLabel: string;
  /**
   * The key of the stateful tool associated with this channel.
   */
  statefulToolKey: string;
};

export type ObserveEdge = Node & StateGraphEdge & {
  __typename?: 'ObserveEdge';
  /**
   * The ID of the node where the edge starts
   */
  from: string;
  /**
   * The unique identifier for the edge
   */
  id: string;
  /**
   * The Y-coordinate of the control point for curved edges
   */
  inflectionY: number | null;
  /**
   * The globally unique ID for the Observe Edge
   */
  nodeId: string;
  /**
   * The ID of the node where the edge ends
   */
  to: string;
  /**
   * The type of the edge
   */
  type: EdgeType;
};

export type PageInfo = {
  __typename?: 'PageInfo';
  endCursor: string | null;
  hasNextPage: boolean;
  hasPreviousPage: boolean;
  startCursor: string | null;
};

export type PaymentFailure = {
  __typename?: 'PaymentFailure';
  /**
   * The amount due on the payment.
   */
  amount: BigNumber;
  /**
   * The date on which the payment failed.
   */
  date: Date;
  /**
   * The Orb invoice id for this payment.
   */
  invoiceId: string;
};

export type PaymentInfo = {
  __typename?: 'PaymentInfo';
  brand: string;
  expMonth: number;
  expYear: number;
  last4: string;
};

export type PhoneNumber = unknown;

export type PlanCadence = 'Annual' | 'Monthly' | '%future added value';

export type PlanType = 'FreeMonthly' | 'MedicalScribeTrial' | 'ProMonthly' | 'Trial' | '%future added value';

export type Query = {
  __typename?: 'Query';
  /**
   * This is only used for testing the scalars provided by this schema.
   */
  _scalars: ScalarsTest;
  _schemaExamples: SchemaExamples;
  me: Identity;
  node: Node | null;
  nodes: Array<Node | null>;
  /**
   * Returns paginated lindy templates based on the query arguments
   */
  templates: QueryTemplatesConnection;
  /**
   * Returns agent definitions grouped by the provided categories
   */
  templatesByCategory: Array<TemplatesByCategoryOutput>;
  /**
   * Retrieves a paginated list of tools available for the authenticated user.
   */
  tools: QueryToolsConnection;
};

export type QueryTemplatesConnection = {
  __typename?: 'QueryTemplatesConnection';
  edges: Array<QueryTemplatesConnectionEdge | null>;
  pageInfo: PageInfo;
  totalCount: number;
};

export type QueryTemplatesConnectionEdge = {
  __typename?: 'QueryTemplatesConnectionEdge';
  cursor: string;
  node: AgentTemplateDefinition;
};

export type QueryToolsConnection = {
  __typename?: 'QueryToolsConnection';
  edges: Array<QueryToolsConnectionEdge | null>;
  pageInfo: PageInfo;
};

export type QueryToolsConnectionEdge = {
  __typename?: 'QueryToolsConnectionEdge';
  cursor: string;
  node: Tool;
};

export type RenameConversationInput = {
  __typename?: 'RenameConversationInput';
  conversationId: string;
  title: string;
};

export type RenameConversationOutput = {
  __typename?: 'RenameConversationOutput';
  conversation: Conversation;
};

export type ReplaceAgentMemoryInput = {
  __typename?: 'ReplaceAgentMemoryInput';
  agentDefinitionId: string;
  memories: Array<MemoryInput>;
};

export type ReplaceAgentMemoryOutput = {
  __typename?: 'ReplaceAgentMemoryOutput';
  agent: AgentDefinition;
};

export type RequestIdentityEmailUpdateInput = {
  __typename?: 'RequestIdentityEmailUpdateInput';
  id: string;
  newEmail: string;
  password: string;
  previousEmail: string;
};

export type RequestIdentityEmailUpdateOutput = {
  __typename?: 'RequestIdentityEmailUpdateOutput';
  identity: Identity;
};

export type ResendIdentityEmailUpdateInput = {
  __typename?: 'ResendIdentityEmailUpdateInput';
  email: string;
  id: string;
  name: string;
};

export type ResendIdentityEmailUpdateOutput = {
  __typename?: 'ResendIdentityEmailUpdateOutput';
  identity: Identity;
};

export type RetryStateGraphTestRunExecutionInput = {
  __typename?: 'RetryStateGraphTestRunExecutionInput';
  /**
   * This is the ID of the agent definition that is being tested
   */
  agentDefinitionId: string;
  /**
   * This list of edges of the state graph
   */
  edges: Array<StateGraphEdgeInput>;
  /**
   * The ID of the execution to retry.
   */
  executionId: string;
  /**
   * The list of nodes of the state graph. Keep in mind this is unrelated to Relay nodes.
   */
  nodes: Array<StateGraphNodeInput>;
  /**
   * The ID of the test run to be updated
   */
  stateGraphTestRunId: string | null;
};

export type RetryStateGraphTestRunExecutionOutput = {
  __typename?: 'RetryStateGraphTestRunExecutionOutput';
  agentDefinition: AgentDefinition;
  /**
   * This will contain the generated Agent Message (if any)
   */
  agentMessage: AgentMessage | null;
  stateGraphTestRun: StateGraphTestRun;
};

export type ScalarsTest = {
  __typename?: 'ScalarsTest';
  jsonObject: JSONObject | null;
  jsonObjectList: Array<JSONObject>;
};

export type SchemaExamples = {
  __typename?: 'SchemaExamples';
  throwErrorWithErrorPlugin: SchemaExamplesThrowErrorWithErrorPluginResult;
  throwErrorWithoutErrorPlugin: string;
};

export type SchemaExamplesThrowErrorWithErrorPluginResult = BaseError | NotFoundError | SchemaExamplesThrowErrorWithErrorPluginSuccess | UnauthorizedError | ValidationError;

export type SchemaExamplesThrowErrorWithErrorPluginSuccess = {
  __typename?: 'SchemaExamplesThrowErrorWithErrorPluginSuccess';
  data: string;
};

export type SelectStateGraphTestRunTriggerEntryPointPayloadInput = {
  __typename?: 'SelectStateGraphTestRunTriggerEntryPointPayloadInput';
  /**
   * This is the ID of the agent definition that is being tested
   */
  agentDefinitionId: string;
  /**
   * The ID of the external payload to start the test run task for
   */
  externalPayloadId: string | null;
  /**
   * The ID of the run attempt to start the test run task for
   */
  runAttemptId: string | null;
  /**
   * The ID of the test run to be updated
   */
  stateGraphTestRunId: string | null;
};

export type SelectStateGraphTestRunTriggerEntryPointPayloadOutput = {
  __typename?: 'SelectStateGraphTestRunTriggerEntryPointPayloadOutput';
  agentDefinition: AgentDefinition;
  /**
   * This will contain the generated Agent Message (if any)
   */
  agentMessage: AgentMessage | null;
  stateGraphTestRun: StateGraphTestRun;
};

export type SkipWakeUpTaskTriggerInput = {
  __typename?: 'SkipWakeUpTaskTriggerInput';
  /**
   * This is the payload of the Trigger, if any. This will be validated against the trigger.
   */
  payload: JSON | null;
  /**
   * This is the ID of the Trigger
   */
  triggerId: string;
};

export type SkipWakeUpTaskTriggerOutput = {
  __typename?: 'SkipWakeUpTaskTriggerOutput';
  agentMessage: AgentMessage;
  conversation: Conversation;
  trigger: Trigger;
};

export type SpeechToTextTerms = 'English' | 'MultiLingual' | '%future added value';

export type StandardEdge = Node & StateGraphEdge & {
  __typename?: 'StandardEdge';
  /**
   * The condition of the edge
   */
  condition: string | null;
  /**
   * The ID of the node where the edge starts
   */
  from: string;
  /**
   * The unique identifier for the edge
   */
  id: string;
  /**
   * The Y-coordinate of the control point for curved edges
   */
  inflectionY: number | null;
  /**
   * The label of the edge
   */
  label: string | null;
  /**
   * The globally unique ID for the Standard Edge
   */
  nodeId: string;
  /**
   * The ID of the node where the edge ends
   */
  to: string;
  /**
   * The type of the edge
   */
  type: EdgeType;
};

export type StartConversationFromNodeInput = {
  __typename?: 'StartConversationFromNodeInput';
  agentDefinitionId: string;
  identityId: string;
  nodeId: string;
  timeZone: string;
};

export type StartConversationFromNodeOutput = {
  __typename?: 'StartConversationFromNodeOutput';
  conversation: Conversation;
};

export type StartLiveCaptureForStateGraphTestRunInput = {
  __typename?: 'StartLiveCaptureForStateGraphTestRunInput';
  /**
   * This is the ID of the agent definition that is being tested
   */
  agentDefinitionId: string;
  /**
   * This list of edges of the state graph
   */
  edges: Array<StateGraphEdgeInput>;
  /**
   * The list of nodes of the state graph. Keep in mind this is unrelated to Relay nodes.
   */
  nodes: Array<StateGraphNodeInput>;
  /**
   * The ID of the entry point trigger node to listen to events for
   */
  triggerNodeId: string;
};

export type StartLiveCaptureForStateGraphTestRunOutput = {
  __typename?: 'StartLiveCaptureForStateGraphTestRunOutput';
  agentDefinition: AgentDefinition;
  /**
   * This will contain the generated Agent Message (if any)
   */
  agentMessage: AgentMessage | null;
  /**
   * This will contain the generated live capture trigger
   */
  liveCaptureTrigger: Trigger;
  stateGraphTestRun: StateGraphTestRun;
};

/**
 * The State Graph contains the entities that are the foundation of a Lindy. The graph has a representation of the expected agent's behavior.
 */
export type StateGraph = Node & {
  __typename?: 'StateGraph';
  /**
   * The conversation entry point node if the state graph has one
   */
  conversationNode: EntryPointConversationNode | null;
  /**
   * The edges of the State Graph
   */
  edges: Array<StateGraphEdge>;
  id: string;
  /**
   * The globally unique ID for the resource
   */
  nodeId: string;
  /**
   * The nodes of the State Graph
   */
  nodes: Array<StateGraphNode>;
  /**
   * The owner of the State Graph
   */
  owner: Identity;
  /**
   * The sticky notes of the State Graph
   */
  stickyNotes: Array<StateGraphStickyNote>;
  /**
   * The version of the State Graph
   */
  version: number;
  /**
   * The start nodes of the workflow
   */
  workflowStartNodes: Array<StateGraphNode>;
};

export type StateGraphAction = {
  __typename?: 'StateGraphAction';
  /**
   * The action details of the action
   */
  action: ActionDetails;
  /**
   * The action configuration of the action
   */
  actionConfiguration: Array<ActionConfigurationMember>;
  /**
   * The ID of the node that created the action
   */
  createdByNodeId: string | null;
  /**
   * The observable channel of the action
   */
  observableChannel: ObservableChannel | null;
  /**
   * Whether the action requires confirmation
   */
  requireConfirmation: AgentRequireConfirmation;
  /**
   * The stateful tool key of the action
   */
  statefulToolKey: string | null;
  /**
   * The tool of the action
   */
  tool: Tool;
};

export type StateGraphEdge = {
  from: string;
  id: string;
  inflectionY: number | null;
  to: string;
  type: EdgeType;
};

export type StateGraphEdgeInput = unknown;

export type StateGraphEntityType = 'Edge' | 'Graph' | 'Node' | 'StickyNote' | '%future added value';

export type StateGraphIssueLevel = 'Error' | 'Warning' | '%future added value';

export type StateGraphIssueType = 'Authorize' | 'Configure' | 'Reference' | 'Structure' | 'Unknown' | '%future added value';

export type StateGraphNode = {
  displayName: string | null;
  id: string;
  manuallyPositioned: boolean;
  position: NodePosition;
  type: NodeType;
};

export type StateGraphNodeInput = unknown;

export type StateGraphStickyNote = Node & {
  __typename?: 'StateGraphStickyNote';
  /**
   * The unique identifier for the sticky note
   */
  id: string;
  /**
   * The globally unique ID for the Sticky Note
   */
  nodeId: string;
  /**
   * The content of the sticky note
   */
  note: string;
  /**
   * The position of the sticky note
   */
  position: StickyNotePosition;
};

export type StateGraphTestRun = Node & {
  __typename?: 'StateGraphTestRun';
  /**
   * This is the Conversation currently being used to test the State Graph. This can be null when the test is waiting for the initial trigger to fire, such as during Live Capturing.
   */
  conversation: Conversation | null;
  greetingMessage: string | null;
  id: string;
  initiallySelectedNodeId: string | null;
  /**
   * This is the current Live Capture trigger, if it exists.
   */
  liveCaptureTrigger: Trigger | null;
  /**
   * The globally unique ID used by Relay
   */
  nodeId: string;
  owner: Identity;
};

export type StateGraphValidationConfigureIssue = StateGraphValidationIssueInterface & {
  __typename?: 'StateGraphValidationConfigureIssue';
  entityType: StateGraphEntityType;
  field: string;
  id: string;
  label: string | null;
  level: StateGraphIssueLevel;
  message: string;
  type: StateGraphIssueType;
};

export type StateGraphValidationError = Error & {
  __typename?: 'StateGraphValidationError';
  issues: Array<StateGraphValidationIssueInterface>;
  message: string;
};

export type StateGraphValidationIssue = StateGraphValidationIssueInterface & {
  __typename?: 'StateGraphValidationIssue';
  entityType: StateGraphEntityType;
  id: string;
  label: string | null;
  level: StateGraphIssueLevel;
  message: string;
  type: StateGraphIssueType;
};

export type StateGraphValidationIssueInterface = {
  entityType: StateGraphEntityType;
  id: string;
  label: string | null;
  level: StateGraphIssueLevel;
  message: string;
  type: StateGraphIssueType;
};

export type StickyNotePosition = {
  __typename?: 'StickyNotePosition';
  x: number;
  y: number;
};

export type StopLiveCaptureForStateGraphTestRunInput = {
  __typename?: 'StopLiveCaptureForStateGraphTestRunInput';
  /**
   * This is the ID of the agent definition that is being tested
   */
  agentDefinitionId: string;
};

export type StopLiveCaptureForStateGraphTestRunOutput = {
  __typename?: 'StopLiveCaptureForStateGraphTestRunOutput';
  agentDefinition: AgentDefinition;
  /**
   * This will contain the generated Agent Message (if any)
   */
  agentMessage: AgentMessage | null;
  stateGraphTestRun: StateGraphTestRun;
};

export type SubscriptionFeatures = {
  __typename?: 'SubscriptionFeatures';
  arePremiumActionsEnabled: boolean;
  arePremiumFeaturesEnabled: boolean;
  isEligibleForFreePlan: boolean;
  speechToText: SpeechToTextTerms | null;
  trialCreditUsageType: TrialCreditUsageType | null;
};

export type SubscriptionInfo = {
  __typename?: 'SubscriptionInfo';
  billingPeriodEndsAt: Date | null;
  billingPeriodStartAt: Date | null;
  billingVersion: BillingVersion;
  creditsInfo: CreditsInfo;
  features: SubscriptionFeatures | null;
  id: string;
  /**
   * Details on last payment failure if there was one.
   */
  lastPaymentFailure: PaymentFailure | null;
  paymentInfo: PaymentInfo | null;
  /**
   * The ID of the plan in Lindy
   */
  planId: string;
  /**
   * The portal/stripe URL
   */
  portal: URL | null;
  subscriptionExpiresAt: Date | null;
  /**
   * The subscription plan
   */
  subscriptionPlan: SubscriptionPlan;
  subscriptionStartAt: Date | null;
  trialInfo: TrialInfo | null;
  /**
   * The upcoming subscription plan if available
   */
  upcomingPlan: UpcomingPlan | null;
};

export type SubscriptionPlan = {
  __typename?: 'SubscriptionPlan';
  cadence: PlanCadence;
  creditAllocation: number;
  id: string;
  name: string;
  planType: PlanType | null;
  price: BigNumber;
};

export type TemplateCategory = 'Emails' | 'Meetings' | 'PersonalAssistant' | 'SalesMarketing' | 'Scribe' | 'Support' | 'WorkflowAutomation' | '%future added value';

export type TemplatesByCategoryOutput = {
  __typename?: 'TemplatesByCategoryOutput';
  category: TemplateCategory;
  featured: Array<AgentTemplateDefinition>;
  templates: Array<AgentTemplateDefinition>;
};

export type TokenAuthConfig = AuthConfig & {
  __typename?: 'TokenAuthConfig';
  /**
   * The method of the auth configuration
   */
  method: AuthMethod;
  /**
   * The provider of the auth configuration
   */
  provider: string;
};

export type Tool = Node & {
  __typename?: 'Tool';
  actions: ToolActionsConnection;
  description: string;
  displayName: string;
  isPopular: boolean;
  nodeId: string;
  tool: LindyTool;
  triggers: ToolTriggersConnection;
};

export type ToolActionsConnection = {
  __typename?: 'ToolActionsConnection';
  edges: Array<ToolActionsConnectionEdge | null>;
  pageInfo: PageInfo;
};

export type ToolActionsConnectionEdge = {
  __typename?: 'ToolActionsConnectionEdge';
  cursor: string;
  node: ActionDetails;
};

export type ToolTriggerChannel = {
  __typename?: 'ToolTriggerChannel';
  /**
   * The key of the stateful tool associated with this channel.
   */
  statefulToolKey: string;
};

export type ToolTriggersConnection = {
  __typename?: 'ToolTriggersConnection';
  edges: Array<ToolTriggersConnectionEdge | null>;
  pageInfo: PageInfo;
};

export type ToolTriggersConnectionEdge = {
  __typename?: 'ToolTriggersConnectionEdge';
  cursor: string;
  node: TriggerDefinition;
};

export type TrialCreditUsageType = 'Restricted' | 'Unrestricted' | '%future added value';

export type TrialInfo = {
  __typename?: 'TrialInfo';
  expiresAt: Date;
};

export type Trigger = Node & {
  __typename?: 'Trigger';
  /**
   * The message to show when live capturing is active, to instruct the user on how to fire the event.
   */
  activeLiveCapturingInstructions: string;
  definition: TriggerDefinition;
  /**
   * Whether the trigger is enabled
   */
  enabled: boolean;
  id: string;
  /**
   * Whether the trigger is configured
   */
  isConfigured: boolean;
  /**
   * Whether the trigger is pending verification
   */
  isPendingVerification: boolean;
  /**
   * The globally unique ID for the resource, this can be used to fetch this resource using the node resolvers.
   */
  nodeId: string;
  owner: Identity;
  /**
   * The trigger's run attempts, i.e. any time we've received a payload for this trigger, whether it was successful or not
   */
  runAttempts: TriggerRunAttemptsConnection;
  /**
   * The setup of the trigger
   */
  setup: TriggerSetup;
  /**
   * A list of possible payloads that can be used to test the trigger if the trigger implements the 'testingPayloads' interface
   */
  testingPayloads: TriggerTestingPayloadsConnection;
};

export type TriggerDefinition = Node & {
  __typename?: 'TriggerDefinition';
  authConfig: AuthConfig | null;
  /**
   * The channel of the trigger
   */
  channel: ToolTriggerChannel | null;
  /**
   * The description of a trigger, to be shown in the frontend
   */
  description: string | null;
  /**
   * The display name of a trigger, to be shown in the frontend
   */
  displayName: string;
  /**
   * The schema of the event that the trigger listens to
   */
  eventSchema: JSONObject;
  /**
   * To properly resolve the fields we need an Identity and [optionally] an specific Auth, this is because the fields themselves may depend on the Identity [and Auth].
   */
  fields: Array<TriggerFieldInterface>;
  /**
   * Allows trigger configurations to be published with marketplace templates.
   */
  hasPublishableConfig: boolean;
  id: string;
  /**
   * Marks this trigger as high throughput, allowing it to run more frequently and be identified as expensive in the UI.
   */
  isHighThroughput: boolean;
  /**
   * Whether the trigger is premium
   */
  isPremium: boolean;
  name: string;
  nodeId: string;
  /**
   * The schema of the raw trigger payload, as it's received from the webhook for example
   */
  payloadSchema: JSONSchema | null;
  /**
   * Whether the trigger supports an empty payload
   */
  supportsEmptyPayload: boolean;
  /**
   * The testing policy of the trigger
   */
  testingPolicy: TriggerTestingPolicy;
  /**
   * The tool of the trigger
   */
  tool: Tool;
};

export type TriggerFieldCheckbox = TriggerFieldInterface & {
  __typename?: 'TriggerFieldCheckbox';
  canBeSharedWithTemplate: boolean;
  defaultValue: boolean | null;
  description: string | null;
  fields: Array<string>;
  id: string;
  isRequired: boolean;
  label: string;
  placeholder: string | null;
  tooltip: string | null;
  type: string;
};

export type TriggerFieldComparableText = TriggerFieldInterface & {
  __typename?: 'TriggerFieldComparableText';
  canBeSharedWithTemplate: boolean;
  description: string | null;
  fields: Array<string>;
  id: string;
  isRequired: boolean;
  label: string;
  placeholder: string | null;
  tooltip: string | null;
  type: string;
};

export type TriggerFieldDateTime = TriggerFieldInterface & {
  __typename?: 'TriggerFieldDateTime';
  canBeSharedWithTemplate: boolean;
  defaultValue: DateTime | null;
  description: string | null;
  fields: Array<string>;
  id: string;
  isRequired: boolean;
  label: string;
  placeholder: string | null;
  tooltip: string | null;
  type: string;
};

export type TriggerFieldInterface = {
  canBeSharedWithTemplate: boolean;
  description: string | null;
  fields: Array<string>;
  id: string;
  isRequired: boolean;
  label: string;
  placeholder: string | null;
  tooltip: string | null;
  type: string;
};

export type TriggerFieldLabel = TriggerFieldInterface & {
  __typename?: 'TriggerFieldLabel';
  canBeSharedWithTemplate: boolean;
  description: string | null;
  fields: Array<string>;
  id: string;
  isRequired: boolean;
  label: string;
  placeholder: string | null;
  tooltip: string | null;
  type: string;
};

export type TriggerFieldLindyEmbedConfig = TriggerFieldInterface & {
  __typename?: 'TriggerFieldLindyEmbedConfig';
  canBeSharedWithTemplate: boolean;
  description: string | null;
  fields: Array<string>;
  id: string;
  isRequired: boolean;
  label: string;
  placeholder: string | null;
  tooltip: string | null;
  type: string;
};

export type TriggerFieldLindyMail = TriggerFieldInterface & {
  __typename?: 'TriggerFieldLindyMail';
  canBeSharedWithTemplate: boolean;
  defaultValue: string | null;
  description: string | null;
  fields: Array<string>;
  id: string;
  isRequired: boolean;
  label: string;
  placeholder: string | null;
  tooltip: string | null;
  type: string;
};

export type TriggerFieldLindyWebhook = TriggerFieldInterface & {
  __typename?: 'TriggerFieldLindyWebhook';
  canBeSharedWithTemplate: boolean;
  defaultValue: string | null;
  description: string | null;
  fields: Array<string>;
  id: string;
  isRequired: boolean;
  label: string;
  placeholder: string | null;
  tooltip: string | null;
  type: string;
};

export type TriggerFieldMultiSelect = TriggerFieldInterface & {
  __typename?: 'TriggerFieldMultiSelect';
  canBeSharedWithTemplate: boolean;
  defaultValue: Array<string> | null;
  description: string | null;
  fields: Array<string>;
  hasAllowAllOption: boolean | null;
  id: string;
  isRequired: boolean;
  label: string;
  /**
   * To properly resolve the options of this field we need an Identity and [optionally] a specific Auth, this is because fetching the options usually resolve around fetching external resources, which may depend on the Identity [and Auth].
   */
  options: Array<TriggerFieldOption>;
  placeholder: string | null;
  tooltip: string | null;
  type: string;
};

export type TriggerFieldNumber = TriggerFieldInterface & {
  __typename?: 'TriggerFieldNumber';
  canBeSharedWithTemplate: boolean;
  defaultValue: number | null;
  description: string | null;
  fields: Array<string>;
  id: string;
  isRequired: boolean;
  label: string;
  max: number | null;
  min: number | null;
  placeholder: string | null;
  step: number | null;
  tooltip: string | null;
  type: string;
};

export type TriggerFieldOption = {
  __typename?: 'TriggerFieldOption';
  id: string;
  label: string;
  value: string;
};

export type TriggerFieldRecurringSchedule = TriggerFieldInterface & {
  __typename?: 'TriggerFieldRecurringSchedule';
  canBeSharedWithTemplate: boolean;
  defaultValue: string | null;
  description: string | null;
  fields: Array<string>;
  id: string;
  isRequired: boolean;
  label: string;
  placeholder: string | null;
  tooltip: string | null;
  type: string;
};

export type TriggerFieldSelect = TriggerFieldInterface & {
  __typename?: 'TriggerFieldSelect';
  canBeSharedWithTemplate: boolean;
  defaultValue: string | null;
  description: string | null;
  fields: Array<string>;
  id: string;
  isRequired: boolean;
  label: string;
  /**
   * To properly resolve the options of this field we need an Identity and [optionally] a specific Auth, this is because fetching the options usually resolve around fetching external resources, which may depend on the Identity [and Auth].
   */
  options: Array<TriggerFieldOption>;
  placeholder: string | null;
  selectFirstByDefault: boolean | null;
  tooltip: string | null;
  type: string;
};

export type TriggerFieldText = TriggerFieldInterface & {
  __typename?: 'TriggerFieldText';
  canBeSharedWithTemplate: boolean;
  defaultValue: string | null;
  description: string | null;
  fields: Array<string>;
  id: string;
  isRequired: boolean;
  label: string;
  placeholder: string | null;
  tooltip: string | null;
  type: string;
};

export type TriggerRunAttempt = Node & {
  __typename?: 'TriggerRunAttempt';
  /**
   * The date and time when the Trigger Run Attempt was created
   */
  createdAt: string;
  /**
   * The external ID of the payload that created this trigger run attempt, if it exists
   */
  createdByExternalPayloadId: string | null;
  id: string;
  llmSummary: string;
  /**
   * The globally unique ID for the Trigger Run Attempt, this can be used to retrieve the Trigger Run Attempt using the node resolvers
   */
  nodeId: string;
  owner: Identity;
  payload: Unknown | null;
  payloadId: string | null;
  processedPayload: Unknown | null;
  shouldRunCheck: TriggerShouldRunCheck | null;
};

export type TriggerRunAttemptsConnection = {
  __typename?: 'TriggerRunAttemptsConnection';
  edges: Array<TriggerRunAttemptsConnectionEdge | null>;
  pageInfo: PageInfo;
};

export type TriggerRunAttemptsConnectionEdge = {
  __typename?: 'TriggerRunAttemptsConnectionEdge';
  cursor: string;
  node: TriggerRunAttempt;
};

export type TriggerSetup = {
  __typename?: 'TriggerSetup';
  actionArgs: JSONObject;
  actionName: string;
  /**
   * The auth of the trigger
   */
  auth: Auth | null;
  toolName: string;
};

export type TriggerShouldRunCheck = {
  __typename?: 'TriggerShouldRunCheck';
  reasons: Array<TriggerShouldRunCheckReason> | null;
  shouldRun: boolean;
  softFilter: boolean | null;
};

export type TriggerShouldRunCheckReason = {
  __typename?: 'TriggerShouldRunCheckReason';
  message: string;
};

export type TriggerTestingPayload = Node & {
  __typename?: 'TriggerTestingPayload';
  externalId: string;
  llmSummary: string;
  /**
   * The globally unique ID for the Trigger Testing Payload, this can be used to retrieve the Trigger Run Attempt using the node resolvers
   */
  nodeId: string;
  payload: Unknown;
  shouldRunCheck: TriggerShouldRunCheck | null;
};

export type TriggerTestingPayloadsConnection = {
  __typename?: 'TriggerTestingPayloadsConnection';
  edges: Array<TriggerTestingPayloadsConnectionEdge | null>;
  pageInfo: PageInfo;
};

export type TriggerTestingPayloadsConnectionEdge = {
  __typename?: 'TriggerTestingPayloadsConnectionEdge';
  cursor: string;
  node: TriggerTestingPayload;
};

export type TriggerTestingPolicy = {
  __typename?: 'TriggerTestingPolicy';
  fullyProcessLiveCapturePayloads: boolean;
  supportsSyntheticPayload: boolean;
};

export type URL = unknown;

export type UnauthorizedError = Error & {
  __typename?: 'UnauthorizedError';
  message: string;
};

export type Unknown = unknown;

export type UpcomingPlan = {
  __typename?: 'UpcomingPlan';
  startDate: Date;
  subscriptionPlan: SubscriptionPlan;
};

export type UpdateAgentFolderInput = {
  __typename?: 'UpdateAgentFolderInput';
  /**
   * This is the ID of the agent folder
   */
  agentFolderId: string;
  /**
   * This is the ID of the owner of the agent folder
   */
  id: string;
  isExpanded: boolean | null;
  name: string | null;
};

export type UpdateAgentFolderOutput = {
  __typename?: 'UpdateAgentFolderOutput';
  agentFolder: AgentFolder;
  owner: Identity;
};

export type UpdateAgentInput = {
  __typename?: 'UpdateAgentInput';
  /**
   * This is the ID of the agent
   */
  agentDefinitionId: string;
  askForConfirmation: boolean | null;
  enabled: boolean | null;
  icon: AgentIconInput | null;
  /**
   * This is the ID of the owner of the agent
   */
  id: string;
  instructions: string | null;
  isFavorite: boolean | null;
  isMuted: boolean | null;
  memories: Array<MemoryInput> | null;
  model: string | null;
  name: string | null;
};

export type UpdateAgentOutput = {
  __typename?: 'UpdateAgentOutput';
  agent: AgentDefinition;
};

export type UpdateCurrentAgentDefinitionInput = {
  __typename?: 'UpdateCurrentAgentDefinitionInput';
  agentDefinitionId: string;
  identityId: string;
};

export type UpdateCurrentAgentDefinitionOutput = {
  __typename?: 'UpdateCurrentAgentDefinitionOutput';
  identity: Identity | null;
};

export type UpdateIdentityAvatarInput = {
  __typename?: 'UpdateIdentityAvatarInput';
  /**
   * The upload data and file metadata. If null the user's avatar will be deleted
   */
  file: UploadAvatarFileInput | null;
  id: string;
};

export type UpdateIdentityAvatarOutput = {
  __typename?: 'UpdateIdentityAvatarOutput';
  identity: Identity;
};

export type UpdateIdentityInput = {
  __typename?: 'UpdateIdentityInput';
  id: string;
  name: string | null;
};

export type UpdateIdentityOutput = {
  __typename?: 'UpdateIdentityOutput';
  identity: Identity;
};

export type UpdateIdentityPasswordInput = {
  __typename?: 'UpdateIdentityPasswordInput';
  email: string;
  newPassword: string;
  previousPassword: string;
};

export type UpdateIdentityPasswordOutput = {
  __typename?: 'UpdateIdentityPasswordOutput';
  identity: Identity;
};

export type UpdateOrCreateCustomAuthInput = {
  __typename?: 'UpdateOrCreateCustomAuthInput';
  /**
   * The ID of the auth, can be omitted
   */
  authId: string | null;
  /**
   * The ID of the conversation to unblock, can be omitted
   */
  conversationId: string | null;
  /**
   * The fields of the auth
   */
  fields: JSONObject;
  /**
   * The provider of the auth
   */
  provider: string;
};

export type UpdateOrCreateCustomAuthOutput = {
  __typename?: 'UpdateOrCreateCustomAuthOutput';
  auth: Auth;
};

export type UpdateOrCreateTokenAuthInput = {
  __typename?: 'UpdateOrCreateTokenAuthInput';
  /**
   * The ID of the auth, can be omitted
   */
  authId: string | null;
  /**
   * The ID of the conversation to unblock, can be omitted
   */
  conversationId: string | null;
  /**
   * The provider of the auth
   */
  provider: string;
  /**
   * The token of the auth
   */
  token: string;
};

export type UpdateOrCreateTokenAuthOutput = {
  __typename?: 'UpdateOrCreateTokenAuthOutput';
  auth: Auth;
};

export type Upload = unknown;

export type UploadAvatarFileInput = {
  __typename?: 'UploadAvatarFileInput';
  heightPx: number;
  upload: Upload;
  widthPx: number;
};

export type ValidationError = Error & {
  __typename?: 'ValidationError';
  fieldErrors: Array<ValidationFieldError>;
  message: string;
};

export type ValidationFieldError = {
  __typename?: 'ValidationFieldError';
  message: string;
  path: Array<string>;
};

