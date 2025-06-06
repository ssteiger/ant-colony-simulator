import {
  ClipboardListIcon,
  DatabaseIcon,
  FileIcon,
  HelpCircleIcon,
  LayoutDashboardIcon,
  MapIcon,
  PlayIcon,
  SearchIcon,
  SettingsIcon,
  BarChart3Icon,
  BugIcon,
} from 'lucide-react'
import { Link } from '@tanstack/react-router'

import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from '~/lib/components/ui/sidebar'
import { NavDocuments } from '~/lib/components/ui/nav-documents'
import { NavMain } from '~/lib/components/ui/nav-main'
import { NavSecondary } from '~/lib/components/ui/nav-secondary'
import { NavUser } from '~/lib/components/ui/nav-user'

interface NavigationItem {
  href: string
  label: string
}

interface AppSidebarProps {
  variant?: 'inset' | 'sidebar' | 'floating'
  navigationItems?: NavigationItem[]
}

const data = {
  user: {
    name: 'Researcher',
    email: 'researcher@antlab.com',
    avatar: '/avatars/researcher.jpg',
  },
  navMain: [
    {
      title: 'Dashboard',
      url: '/dashboard',
      icon: LayoutDashboardIcon,
    },
    {
      title: 'Simulation',
      url: '/simulation',
      icon: PlayIcon,
    },
    {
      title: 'Colonies',
      url: '/colonies',
      icon: BugIcon,
    },
    {
      title: 'Environment',
      url: '/environment',
      icon: MapIcon,
    },
    {
      title: 'Analytics',
      url: '/analytics',
      icon: BarChart3Icon,
    },
  ],
  navSecondary: [
    {
      title: 'Settings',
      url: '/settings',
      icon: SettingsIcon,
    },
    {
      title: 'Help & Documentation',
      url: '/help',
      icon: HelpCircleIcon,
    },
    {
      title: 'Search',
      url: '/search',
      icon: SearchIcon,
    },
  ],
  tools: [],
}

export function AppSidebar({ variant = 'inset' }: AppSidebarProps) {
  return (
    <Sidebar variant={variant}>
      <SidebarHeader>
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton asChild className="data-[slot=sidebar-menu-button]:!p-1.5">
              <Link to="/home">
                <span className="text-base font-semibold">Ant Colony Simulator</span>
              </Link>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarHeader>
      <SidebarContent>
        <NavMain items={data.navMain} />
        <NavDocuments items={data.tools} />
        <NavSecondary items={data.navSecondary} className="mt-auto" />
      </SidebarContent>
      <SidebarFooter>
        <NavUser user={data.user} />
      </SidebarFooter>
    </Sidebar>
  )
}
