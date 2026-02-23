import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from './Card';

describe('Card', () => {
  it('supports compound component API with static subcomponents', () => {
    render(
      <Card>
        <Card.Header>
          <Card.Title>SomeTitle</Card.Title>
          <Card.Description>SomeDescription</Card.Description>
        </Card.Header>
        <Card.Content>SomeContent</Card.Content>
        <Card.Footer>SomeFooter</Card.Footer>
      </Card>
    );

    expect(screen.getByRole('heading', { level: 3, name: 'SomeTitle' })).toBeInTheDocument();
    expect(screen.getByText('SomeDescription')).toBeInTheDocument();
    expect(screen.getByText('SomeContent')).toBeInTheDocument();
    expect(screen.getByText('SomeFooter')).toBeInTheDocument();
  });

  it('renders with default surface styling', () => {
    render(<Card data-testid="card">Body</Card>);

    const card = screen.getByTestId('card');
    expect(card).toHaveClass('rounded-lg');
    expect(card).toHaveClass('border-plugin-border');
    expect(card).toHaveClass('bg-plugin-surface');
    expect(card).toHaveClass('p-4');
  });

  it('renders composable subcomponents with semantic title heading', () => {
    render(
      <Card>
        <CardHeader>
          <CardTitle>Oscillator</CardTitle>
          <CardDescription>Main voicing controls</CardDescription>
        </CardHeader>
        <CardContent>
          <div>Content</div>
        </CardContent>
        <CardFooter>
          <button type="button">Apply</button>
        </CardFooter>
      </Card>
    );

    expect(screen.getByRole('heading', { level: 3, name: 'Oscillator' })).toBeInTheDocument();
    expect(screen.getByText('Main voicing controls')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Apply' })).toBeInTheDocument();
  });

  it('supports custom className overrides on each subcomponent', () => {
    render(
      <Card className="custom-card" data-testid="card">
        <CardHeader className="custom-header" data-testid="header">
          <CardTitle className="custom-title">Title</CardTitle>
          <CardDescription className="custom-description">Description</CardDescription>
        </CardHeader>
        <CardContent className="custom-content" data-testid="content">
          Content
        </CardContent>
        <CardFooter className="custom-footer" data-testid="footer">
          Footer
        </CardFooter>
      </Card>
    );

    expect(screen.getByTestId('card')).toHaveClass('custom-card');
    expect(screen.getByTestId('header')).toHaveClass('custom-header');
    expect(screen.getByText('Title')).toHaveClass('custom-title');
    expect(screen.getByText('Description')).toHaveClass('custom-description');
    expect(screen.getByTestId('content')).toHaveClass('custom-content');
    expect(screen.getByTestId('footer')).toHaveClass('custom-footer');
  });
});
