$pdf_mode = 1;

#https:  // stackoverflow.com/a/6330266
$pdflatex = 'xelatex --shell-escape %O -interaction=nonstopmode %S';

@default_files = ('dist/report.tex');

$out_dir = 'dist';