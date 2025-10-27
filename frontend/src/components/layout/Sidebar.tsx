import { NavLink } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import {
  HomeIcon,
  RectangleStackIcon,
  WrenchScrewdriverIcon,
  ClipboardDocumentListIcon,
  ChatBubbleLeftRightIcon,
  ClockIcon,
  Cog6ToothIcon,
  UserGroupIcon,
} from '@heroicons/react/24/outline';

export const Sidebar: React.FC = () => {
  const { t } = useTranslation();

  const navigation = [
    { name: t('nav.dashboard'), href: '/dashboard', icon: HomeIcon },
    { name: t('nav.agents'), href: '/agents', icon: UserGroupIcon },
    { name: t('nav.flows'), href: '/flows', icon: RectangleStackIcon },
    { name: t('nav.mcpTools'), href: '/mcp/tools', icon: WrenchScrewdriverIcon },
    { name: t('nav.llmConfig'), href: '/config/llm', icon: Cog6ToothIcon },
    { name: t('nav.vectorConfig'), href: '/config/vector', icon: Cog6ToothIcon },
    { name: t('nav.auditLogs'), href: '/audit/logs', icon: ClipboardDocumentListIcon },
    { name: t('nav.executions'), href: '/executions', icon: ClockIcon },
    { name: t('nav.sessions'), href: '/sessions', icon: ChatBubbleLeftRightIcon },
  ];
  return (
    <div className="hidden lg:fixed lg:inset-y-0 lg:flex lg:w-64 lg:flex-col">
      <div className="flex flex-col flex-grow bg-white border-r border-gray-200 pt-5 pb-4 overflow-y-auto">
        <div className="flex items-center flex-shrink-0 px-6">
          <h1 className="text-2xl font-bold text-primary-600">
            {t('nav.agentPlatform')}
          </h1>
        </div>
        
        <nav className="mt-8 flex-1 px-3 space-y-1">
          {navigation.map((item) => (
            <NavLink
              key={item.name}
              to={item.href}
              className={({ isActive }) =>
                `flex items-center gap-3 px-3 py-2 text-sm font-medium rounded-lg transition-colors ${
                  isActive
                    ? 'bg-primary-50 text-primary-600'
                    : 'text-gray-700 hover:bg-gray-50'
                }`
              }
            >
              <item.icon className="w-5 h-5" />
              {item.name}
            </NavLink>
          ))}
        </nav>
      </div>
    </div>
  );
};
