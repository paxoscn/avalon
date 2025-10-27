# Requirements Document

## Introduction

本文档定义了数字人(Agent)实体的需求。数字人是一个可配置的AI助手实体，具有知识库、工具、工作流等资源，可以被用户雇佣使用。每个数字人由创建者拥有和管理，支持复制和共享功能。

## Glossary

- **Agent System**: 数字人管理系统，负责Agent实体的创建、配置、权限管理和雇佣关系
- **Agent**: 数字人实体，代表一个可配置的AI助手
- **Creator**: 创建者，创建Agent的用户
- **Employment Relationship**: 雇佣关系，用户与Agent之间的多对多关联
- **Allocation Relationship**: 分配关系，用户与Agent之间的多对多关联
- **Source Agent**: 来源Agent，被复制的原始Agent
- **Knowledge Base**: 知识库，Agent可访问的向量存储配置
- **MCP Tool**: MCP工具，Agent可使用的工具配置
- **Flow**: 工作流，Agent可执行的流程定义
- **Preset Question**: 预设问题，Agent界面上显示的快速问题选项

## Requirements

### Requirement 1

**User Story:** 作为系统管理员，我希望系统能够存储和管理Agent实体及其所有属性，以便用户可以创建和配置数字人

#### Acceptance Criteria

1. THE Agent System SHALL store Agent entities with tenant_id, name, avatar, system_prompt, additional_settings, source_agent_id, and creator_id attributes
2. THE Agent System SHALL associate each Agent with a list of knowledge base configurations
3. THE Agent System SHALL associate each Agent with a list of MCP tool configurations
4. THE Agent System SHALL associate each Agent with a list of Flow configurations
5. THE Agent System SHALL store up to three preset questions for each Agent

### Requirement 2

**User Story:** 作为用户，我希望创建自己的Agent并配置其属性，以便定制符合我需求的数字人助手

#### Acceptance Criteria

1. WHEN a user creates an Agent, THE Agent System SHALL record the user as the creator_id
2. WHEN a user creates an Agent, THE Agent System SHALL require tenant_id and name as mandatory fields
3. THE Agent System SHALL allow the creator to configure avatar, system_prompt, additional_settings, and preset_questions
4. THE Agent System SHALL allow the creator to associate knowledge bases, MCP tools, and Flows with the Agent
5. THE Agent System SHALL validate that preset_questions contains no more than three items

### Requirement 3

**User Story:** 作为Agent创建者，我希望只有我能修改我创建的Agent配置，以便保护我的Agent不被他人篡改

#### Acceptance Criteria

1. WHEN a user attempts to modify an Agent, THE Agent System SHALL verify the user's ID matches the Agent's creator_id
2. IF the user's ID does not match the creator_id, THEN THE Agent System SHALL reject the modification request with an authorization error
3. THE Agent System SHALL allow the creator to update all Agent attributes including name, avatar, system_prompt, additional_settings, and preset_questions
4. THE Agent System SHALL allow the creator to modify the associated knowledge bases, MCP tools, and Flows
5. THE Agent System SHALL allow the creator to delete the Agent

### Requirement 4

**User Story:** 作为用户，我希望能够复制现有的Agent，以便基于已有配置快速创建新的数字人

#### Acceptance Criteria

1. WHEN a user copies an Agent, THE Agent System SHALL create a new Agent with all configuration attributes copied from the source
2. WHEN a user copies an Agent, THE Agent System SHALL set the source_agent_id to reference the original Agent
3. WHEN a user copies an Agent, THE Agent System SHALL set the creator_id to the copying user's ID
4. THE Agent System SHALL copy all associated knowledge bases, MCP tools, and Flows to the new Agent
5. THE Agent System SHALL allow the new Agent to be modified independently of the source Agent

### Requirement 5

**User Story:** 作为用户，我希望能够雇佣Agent以便使用它们提供的服务

#### Acceptance Criteria

1. THE Agent System SHALL maintain a many-to-many employment relationship between Users and Agents
2. WHEN a user employs an Agent, THE Agent System SHALL create an employment record linking the user and Agent
3. THE Agent System SHALL allow a user to employ multiple Agents
4. THE Agent System SHALL allow an Agent to be employed by multiple users
5. WHEN a user terminates employment, THE Agent System SHALL remove the employment relationship record

### Requirement 6

**User Story:** 作为用户，我希望在界面上以卡片样式查看Agent列表，以便直观地浏览和选择数字人

#### Acceptance Criteria

1. THE Agent System SHALL provide an API endpoint that returns a list of Agents with pagination support
2. THE Agent System SHALL include agent name, avatar, system_prompt preview, and creator information in the list response
3. THE Agent System SHALL indicate whether the current user has employed each Agent in the list
4. THE Agent System SHALL indicate whether the current user has been allocated each Agent in the list
5. THE Agent System SHALL allow filtering Agents by employment status (employed by current user)
6. THE Agent System SHALL allow filtering Agents by allocation status (allocated to current user)
7. THE Agent System SHALL return Agent data in a format suitable for card-style UI rendering

### Requirement 7

**User Story:** 作为用户，我希望查看Agent的详细信息，以便了解其完整配置和能力

#### Acceptance Criteria

1. THE Agent System SHALL provide an API endpoint that returns complete Agent details by ID
2. THE Agent System SHALL include all Agent attributes in the detail response
3. THE Agent System SHALL include the full list of associated knowledge bases with their configurations
4. THE Agent System SHALL include the full list of associated MCP tools with their configurations
5. THE Agent System SHALL include the full list of associated Flows with their configurations
6. IF the Agent has a source_agent_id, THE Agent System SHALL include source Agent reference information
7. THE Agent System SHALL indicate whether the current user is the creator, whether they have employed the Agent and whether they have been allocated the Agent

### Requirement 8

**User Story:** 作为用户，我希望能够分配Agent以便使用它们提供的服务

#### Acceptance Criteria

1. THE Agent System SHALL maintain a many-to-many allocation relationship between Users and Agents
2. WHEN a user allocates an Agent, THE Agent System SHALL create an allocation record linking the user and Agent
3. THE Agent System SHALL allow a user to be allocated multiple Agents
4. THE Agent System SHALL allow an Agent to be allocated to multiple users
5. WHEN a user terminates allocation, THE Agent System SHALL remove the allocation relationship record
